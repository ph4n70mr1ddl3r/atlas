use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Procurement Contracts (Oracle Fusion SCM > Procurement > Contracts)
// ============================================================================

/// Contract type definition (e.g. Blanket Purchase Agreement, Contract Purchase Agreement)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractType {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code for the contract type
    pub code: String,
    /// Display name
    pub name: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Type classification: "blanket", "`purchase_agreement`", "service", "lease", "other"
    pub contract_classification: String,
    /// Whether this contract type requires approval before activation
    pub requires_approval: bool,
    /// Default contract duration in days (optional)
    pub default_duration_days: Option<i32>,
    /// Whether the contract allows amount-based commitments
    pub allow_amount_commitment: bool,
    /// Whether the contract allows quantity-based commitments
    pub allow_quantity_commitment: bool,
    /// Whether contract lines can be added after activation
    pub allow_line_additions: bool,
    /// Whether contract price can be adjusted
    pub allow_price_adjustment: bool,
    /// Whether renewal is allowed
    pub allow_renewal: bool,
    /// Whether termination is allowed
    pub allow_termination: bool,
    /// Maximum number of renewals (None = unlimited)
    pub max_renewals: Option<i32>,
    /// Default payment terms code
    pub default_payment_terms_code: Option<String>,
    /// Default currency code
    pub default_currency_code: Option<String>,
    /// Whether this contract type is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Procurement contract header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcurementContract {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// System-generated contract number
    pub contract_number: String,
    /// Descriptive title
    pub title: String,
    /// Description of the contract
    pub description: Option<String>,
    /// Contract type code
    pub contract_type_code: Option<String>,
    /// Contract classification: "blanket", "`purchase_agreement`", "service", "lease", "other"
    pub contract_classification: String,
    /// Current status: "draft", "`pending_approval`", "active", "expired", "terminated", "closed"
    pub status: String,
    /// Supplier/vendor UUID
    pub supplier_id: Uuid,
    /// Supplier number
    pub supplier_number: Option<String>,
    /// Supplier name
    pub supplier_name: Option<String>,
    /// Supplier contact name
    pub supplier_contact: Option<String>,
    /// Buyer/procurement officer UUID
    pub buyer_id: Option<Uuid>,
    /// Buyer name
    pub buyer_name: Option<String>,
    /// Contract start date
    pub start_date: Option<chrono::NaiveDate>,
    /// Contract end date
    pub end_date: Option<chrono::NaiveDate>,
    /// Total committed amount
    pub total_committed_amount: String,
    /// Total released (ordered) amount against this contract
    pub total_released_amount: String,
    /// Total invoiced amount against this contract
    pub total_invoiced_amount: String,
    /// Currency code
    pub currency_code: String,
    /// Payment terms code
    pub payment_terms_code: Option<String>,
    /// Whether price is fixed or variable
    pub price_type: String,
    /// Number of renewals so far
    pub renewal_count: i32,
    /// Maximum allowed renewals
    pub max_renewals: Option<i32>,
    /// Number of contract lines
    pub line_count: i32,
    /// Number of milestones
    pub milestone_count: i32,
    /// Approver UUID
    pub approved_by: Option<Uuid>,
    /// Approval timestamp
    pub approved_at: Option<DateTime<Utc>>,
    /// Rejection reason (if applicable)
    pub rejection_reason: Option<String>,
    /// Termination reason (if applicable)
    pub termination_reason: Option<String>,
    /// Terminated by
    pub terminated_by: Option<Uuid>,
    /// Termination date
    pub terminated_at: Option<DateTime<Utc>>,
    /// Notes
    pub notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Procurement contract line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent contract
    pub contract_id: Uuid,
    /// Line number within contract
    pub line_number: i32,
    /// Product/item description
    pub item_description: String,
    /// Product/item code or SKU
    pub item_code: Option<String>,
    /// Product category
    pub category: Option<String>,
    /// Unit of measure
    pub uom: Option<String>,
    /// Quantity committed
    pub quantity_committed: Option<String>,
    /// Quantity released (ordered so far)
    pub quantity_released: String,
    /// Unit price
    pub unit_price: String,
    /// Line amount (`quantity_committed` * `unit_price`)
    pub line_amount: String,
    /// Amount released (invoiced/spent so far)
    pub amount_released: String,
    /// Delivery date
    pub delivery_date: Option<chrono::NaiveDate>,
    /// Supplier part number
    pub supplier_part_number: Option<String>,
    /// GL account code
    pub account_code: Option<String>,
    /// Cost center
    pub cost_center: Option<String>,
    /// Project ID
    pub project_id: Option<Uuid>,
    /// Line notes
    pub notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Contract milestone / deliverable
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractMilestone {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent contract
    pub contract_id: Uuid,
    /// Optional parent line
    pub contract_line_id: Option<Uuid>,
    /// Milestone sequence number
    pub milestone_number: i32,
    /// Milestone name/title
    pub name: String,
    /// Detailed description
    pub description: Option<String>,
    /// Milestone type: "delivery", "payment", "review", "acceptance", "custom"
    pub milestone_type: String,
    /// Target completion date
    pub target_date: chrono::NaiveDate,
    /// Actual completion date
    pub actual_date: Option<chrono::NaiveDate>,
    /// Status: "pending", "`in_progress`", "completed", "overdue", "cancelled"
    pub status: String,
    /// Amount associated with this milestone
    pub amount: String,
    /// Percentage of total contract (for progress tracking)
    pub percent_of_total: String,
    /// Deliverable description
    pub deliverable: Option<String>,
    /// Whether this milestone is billable
    pub is_billable: bool,
    /// Approved by
    pub approved_by: Option<Uuid>,
    /// Approved date
    pub approved_at: Option<DateTime<Utc>>,
    /// Notes
    pub notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Contract renewal record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractRenewal {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent contract
    pub contract_id: Uuid,
    /// Renewal number (1 for first renewal, etc.)
    pub renewal_number: i32,
    /// Previous end date
    pub previous_end_date: chrono::NaiveDate,
    /// New end date
    pub new_end_date: chrono::NaiveDate,
    /// Renewal type: "automatic", "manual", "negotiated"
    pub renewal_type: String,
    /// Any terms that changed during renewal
    pub terms_changed: Option<String>,
    /// Renewed by
    pub renewed_by: Option<Uuid>,
    /// Renewal date
    pub renewed_at: DateTime<Utc>,
    /// Notes
    pub notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Contract spend entry (tracks actual spend against contract)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractSpend {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent contract
    pub contract_id: Uuid,
    /// Optional parent contract line
    pub contract_line_id: Option<Uuid>,
    /// Source document type (e.g. "`purchase_order`", "invoice")
    pub source_type: String,
    /// Source document ID
    pub source_id: Option<Uuid>,
    /// Source document number
    pub source_number: Option<String>,
    /// Transaction date
    pub transaction_date: chrono::NaiveDate,
    /// Amount
    pub amount: String,
    /// Quantity (if applicable)
    pub quantity: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Procurement Contracts dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractDashboardSummary {
    /// Total contracts count
    pub total_contracts: i32,
    /// Active contracts count
    pub active_contracts: i32,
    /// Contracts expiring within 30 days
    pub expiring_contracts_count: i32,
    /// Total committed amount across all active contracts
    pub total_committed_amount: String,
    /// Total released/spent amount
    pub total_released_amount: String,
    /// Utilization percentage (released / committed)
    pub utilization_percent: String,
    /// Contracts by status
    pub contracts_by_status: serde_json::Value,
    /// Contracts by type
    pub contracts_by_type: serde_json::Value,
    /// Top suppliers by committed amount
    pub top_suppliers: serde_json::Value,
}

// ════════════════════════════════════════════════════════════════════════════════
// Inventory Management (Oracle Fusion SCM > Inventory Management)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Inventory Management provides:
// - Inventory Organizations: Warehouses, stores, and distribution centers
// - Items: Products, materials, and supplies with full attribute tracking
// - Item Categories: Hierarchical classification of items
// - Subinventories: Logical storage areas within organizations
// - Locators: Specific bins/shelves within subinventories
// - On-Hand Balances: Real-time stock quantities with lot/serial/revision tracking
// - Inventory Transactions: All material movements (receipts, issues, transfers, adjustments)
// - Transaction Types: Configurable transaction type definitions
// - Cycle Counts: Periodic stock verification with variance analysis
// - Transaction Reasons: Coded reasons for material movements
//
// Oracle Fusion equivalent: SCM > Inventory Management

/// Inventory Organization (warehouse, store, distribution center)
/// Oracle Fusion: Inventory > Organizations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryOrganization {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// "warehouse", "store", "`distribution_center`", "manufacturing", "other"
    pub org_type: String,
    pub location_code: Option<String>,
    pub address: Option<serde_json::Value>,
    pub is_active: bool,
    pub default_subinventory_code: Option<String>,
    pub default_currency_code: String,
    pub requires_approval_for_issues: bool,
    pub requires_approval_for_transfers: bool,
    pub enable_lot_control: bool,
    pub enable_serial_control: bool,
    pub enable_revision_control: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Item Category (hierarchical)
/// Oracle Fusion: Inventory > Item Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub track_as_asset: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Item (product, material, supply)
/// Oracle Fusion: Inventory > Items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_code: String,
    pub name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_code: Option<String>,
    /// "inventory", "`non_inventory`", "service", "expense", "capital"
    pub item_type: String,
    pub uom: String,
    pub secondary_uom: Option<String>,
    pub weight: Option<String>,
    pub weight_uom: Option<String>,
    pub volume: Option<String>,
    pub volume_uom: Option<String>,
    pub list_price: String,
    pub standard_cost: String,
    pub min_order_quantity: Option<String>,
    pub max_order_quantity: Option<String>,
    pub lead_time_days: i32,
    pub shelf_life_days: Option<i32>,
    pub is_lot_controlled: bool,
    pub is_serial_controlled: bool,
    pub is_revision_controlled: bool,
    pub is_perishable: bool,
    pub is_hazardous: bool,
    pub is_purchasable: bool,
    pub is_sellable: bool,
    pub is_stockable: bool,
    pub inventory_asset_account_code: Option<String>,
    pub expense_account_code: Option<String>,
    pub cost_of_goods_sold_account: Option<String>,
    pub revenue_account_code: Option<String>,
    pub image_url: Option<String>,
    pub barcode: Option<String>,
    pub supplier_item_codes: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subinventory (logical storage area)
/// Oracle Fusion: Inventory > Subinventories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subinventory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inventory_org_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// "storage", "receiving", "staging", "inspection", "packing", "other"
    pub subinventory_type: String,
    pub asset_subinventory: bool,
    pub quantity_tracked: bool,
    pub location_code: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Locator (bin/shelf/row within a subinventory)
/// Oracle Fusion: Inventory > Locators
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Locator {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subinventory_id: Uuid,
    pub code: String,
    pub description: Option<String>,
    pub picker_order: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// On-Hand Balance (real-time stock quantity)
/// Oracle Fusion: Inventory > On-hand Quantities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnHandBalance {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inventory_org_id: Uuid,
    pub item_id: Uuid,
    pub subinventory_id: Uuid,
    pub locator_id: Option<Uuid>,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub revision: Option<String>,
    pub quantity: String,
    pub reserved_quantity: String,
    pub available_quantity: String,
    pub unit_cost: String,
    pub total_value: String,
    pub last_transaction_date: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inventory Transaction Type
/// Oracle Fusion: Inventory > Transaction Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryTransactionType {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// "receive", "issue", "transfer", "adjustment", "`return_to_vendor`",
    /// "`return_to_customer`", "`cycle_count_adjustment`", "`misc_receipt`", "`misc_issue`"
    pub transaction_action: String,
    /// "manual", "`purchase_order`", "`sales_order`", "`work_order`", "system"
    pub source_type: String,
    pub is_system: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inventory Transaction (material movement)
/// Oracle Fusion: Inventory > Transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transaction_number: String,
    pub transaction_type_id: Option<Uuid>,
    pub transaction_type_code: Option<String>,
    pub transaction_action: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub source_line_id: Option<Uuid>,
    pub item_id: Uuid,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    // From location
    pub from_inventory_org_id: Option<Uuid>,
    pub from_subinventory_id: Option<Uuid>,
    pub from_locator_id: Option<Uuid>,
    // To location
    pub to_inventory_org_id: Option<Uuid>,
    pub to_subinventory_id: Option<Uuid>,
    pub to_locator_id: Option<Uuid>,
    // Quantities
    pub quantity: String,
    pub uom: String,
    pub unit_cost: String,
    pub total_cost: String,
    // Lot/Serial/Revision
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub revision: Option<String>,
    // Dates
    pub transaction_date: DateTime<Utc>,
    pub accounting_date: Option<chrono::NaiveDate>,
    // Reference
    pub reason_id: Option<Uuid>,
    pub reason_name: Option<String>,
    pub reference: Option<String>,
    pub reference_type: Option<String>,
    pub notes: Option<String>,
    // GL
    pub is_posted: bool,
    pub posted_at: Option<DateTime<Utc>>,
    pub journal_entry_id: Option<Uuid>,
    // Workflow
    /// "pending", "approved", "processed", "cancelled"
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    // Audit
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cycle Count Header
/// Oracle Fusion: Inventory > Cycle Counts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleCountHeader {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub count_number: String,
    pub name: String,
    pub description: Option<String>,
    pub inventory_org_id: Uuid,
    pub subinventory_id: Option<Uuid>,
    pub count_date: chrono::NaiveDate,
    /// "draft", "`in_progress`", "completed", "cancelled"
    pub status: String,
    /// "full", "abc", "random", "`by_category`"
    pub count_method: String,
    pub tolerance_percent: String,
    pub total_items: i32,
    pub counted_items: i32,
    pub matched_items: i32,
    pub mismatched_items: i32,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cycle Count Line
/// Oracle Fusion: Inventory > Cycle Count Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleCountLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cycle_count_id: Uuid,
    pub line_number: i32,
    pub item_id: Uuid,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub subinventory_id: Option<Uuid>,
    pub locator_id: Option<Uuid>,
    pub lot_number: Option<String>,
    pub revision: Option<String>,
    pub system_quantity: String,
    pub count_quantity_1: Option<String>,
    pub count_quantity_2: Option<String>,
    pub count_quantity_3: Option<String>,
    pub count_date_1: Option<DateTime<Utc>>,
    pub count_date_2: Option<DateTime<Utc>>,
    pub count_date_3: Option<DateTime<Utc>>,
    pub counted_by_1: Option<Uuid>,
    pub counted_by_2: Option<Uuid>,
    pub counted_by_3: Option<Uuid>,
    pub approved_quantity: Option<String>,
    pub variance_quantity: Option<String>,
    pub variance_percent: Option<String>,
    pub is_matched: bool,
    /// "pending", "counted", "recount", "approved", "adjusted"
    pub status: String,
    pub adjustment_transaction_id: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Transaction Reason
/// Oracle Fusion: Inventory > Transaction Reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReason {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub applicable_actions: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inventory Dashboard Summary
/// Oracle Fusion: Inventory Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryDashboardSummary {
    pub total_items: i32,
    pub active_items: i32,
    pub total_organizations: i32,
    pub total_on_hand_value: String,
    pub total_pending_transactions: i32,
    pub total_processed_transactions: i32,
    pub items_by_type: serde_json::Value,
    pub items_by_category: serde_json::Value,
    pub transactions_by_action: serde_json::Value,
    pub top_items_by_value: serde_json::Value,
    pub pending_cycle_counts: i32,
    pub low_stock_items: i32,
}

// ============================================================================
// Customer Returns Management / Return Material Authorization (RMA)
// Oracle Fusion Cloud ERP: Order Management > Returns
// ============================================================================

/// Return reason code definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnReason {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub return_type: String, // standard_return, exchange, repair, warranty
    pub default_disposition: Option<String>, // return_to_stock, scrap, inspect, repair
    pub requires_approval: bool,
    pub credit_issued_automatically: bool,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Return Material Authorization (RMA) header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnAuthorization {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rma_number: String,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub return_type: String, // standard_return, exchange, repair, warranty
    pub status: String, // draft, submitted, approved, rejected, partially_received, received, closed, cancelled
    pub reason_code: Option<String>,
    pub reason_name: Option<String>,
    pub original_order_number: Option<String>,
    pub original_order_id: Option<Uuid>,
    pub customer_contact: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub return_date: chrono::NaiveDate,
    pub expected_receipt_date: Option<chrono::NaiveDate>,
    pub total_quantity: String,
    pub total_amount: String,
    pub total_credit_amount: String,
    pub currency_code: String,
    pub notes: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub credit_memo_id: Option<Uuid>,
    pub credit_memo_number: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// RMA line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rma_id: Uuid,
    pub line_number: i32,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub original_line_id: Option<Uuid>,
    pub original_quantity: String,
    pub return_quantity: String,
    pub unit_price: String,
    pub return_amount: String,
    pub credit_amount: String,
    pub reason_code: Option<String>,
    pub disposition: Option<String>, // return_to_stock, scrap, inspect, repair, exchange
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub condition: Option<String>, // good, damaged, defective, wrong_item
    pub received_quantity: String,
    pub received_date: Option<chrono::NaiveDate>,
    pub inspection_status: Option<String>, // pending, passed, failed, pending_review
    pub inspection_notes: Option<String>,
    pub credit_status: Option<String>, // pending, issued, reversed
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit memo generated from returns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditMemo {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub credit_memo_number: String,
    pub rma_id: Option<Uuid>,
    pub rma_number: Option<String>,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub status: String, // draft, issued, applied, partially_applied, reversed, cancelled
    pub applied_amount: String,
    pub remaining_amount: String,
    pub issue_date: Option<chrono::NaiveDate>,
    pub applied_to_invoice_id: Option<Uuid>,
    pub applied_to_invoice_number: Option<String>,
    pub gl_account_code: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Customer Returns dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnsDashboardSummary {
    pub total_rmas: i32,
    pub open_rmas: i32,
    pub pending_approval: i32,
    pub pending_receipt: i32,
    pub pending_inspection: i32,
    pub total_credit_issued_amount: String,
    pub total_credit_pending_amount: String,
    pub rmas_by_status: serde_json::Value,
    pub rmas_by_reason: serde_json::Value,
    pub rmas_by_disposition: serde_json::Value,
    pub top_returned_items: serde_json::Value,
    pub average_processing_days: String,
}

// ============================================================================
// Advanced Pricing Management (Oracle Fusion Order Management > Pricing)
// ============================================================================

/// Price List definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceList {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub currency_code: String,
    pub list_type: String,
    pub pricing_basis: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Price List Line (item-level pricing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceListLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub price_list_id: Uuid,
    pub line_number: i32,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub pricing_unit_of_measure: String,
    pub list_price: String,
    pub unit_price: String,
    pub cost_price: String,
    pub margin_percent: String,
    pub minimum_quantity: String,
    pub maximum_quantity: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Price Tier (quantity break)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceTier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub price_list_line_id: Uuid,
    pub tier_number: i32,
    pub from_quantity: String,
    pub to_quantity: Option<String>,
    pub price: String,
    pub discount_percent: String,
    pub price_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Discount Rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: String,
    pub application_method: String,
    pub stacking_rule: String,
    pub priority: i32,
    pub condition: serde_json::Value,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub is_active: bool,
    pub usage_count: i32,
    pub max_usage: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Charge Definition (shipping, handling, surcharges)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargeDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub charge_type: String,
    pub charge_category: String,
    pub calculation_method: String,
    pub charge_amount: String,
    pub charge_percent: String,
    pub minimum_charge: String,
    pub maximum_charge: Option<String>,
    pub taxable: bool,
    pub condition: serde_json::Value,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pricing Strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingStrategy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub strategy_type: String,
    pub priority: i32,
    pub condition: serde_json::Value,
    pub price_list_id: Option<Uuid>,
    pub markup_percent: String,
    pub markdown_percent: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Price Calculation Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalculationLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub calculation_date: DateTime<Utc>,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub line_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub requested_quantity: Option<String>,
    pub unit_list_price: String,
    pub unit_selling_price: String,
    pub discount_amount: String,
    pub discount_rule_id: Option<Uuid>,
    pub charge_amount: String,
    pub charge_definition_id: Option<Uuid>,
    pub strategy_id: Option<Uuid>,
    pub price_list_id: Option<Uuid>,
    pub calculation_steps: serde_json::Value,
    pub currency_code: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Result of a price calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalculationResult {
    pub list_price: String,
    pub discount_amount: String,
    pub charge_amount: String,
    pub unit_selling_price: String,
    pub extended_price: String,
    pub currency_code: String,
    pub applied_discount_rule_code: Option<String>,
    pub applied_charge_code: Option<String>,
    pub applied_price_list_code: Option<String>,
    pub applied_strategy_code: Option<String>,
    pub tier_applied: Option<i32>,
    pub calculation_steps: Vec<PriceCalculationStep>,
}

/// A single step in the pricing calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalculationStep {
    pub step_type: String,
    pub description: String,
    pub amount_before: String,
    pub amount_after: String,
    pub rule_applied: Option<String>,
}

/// Pricing dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingDashboardSummary {
    pub total_price_lists: i32,
    pub active_price_lists: i32,
    pub total_discount_rules: i32,
    pub active_discount_rules: i32,
    pub total_charge_definitions: i32,
    pub total_strategies: i32,
    pub total_calculations_today: i32,
    pub price_lists_by_status: serde_json::Value,
    pub discount_rules_by_type: serde_json::Value,
    pub charges_by_type: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════
// Sales Commission Management
// Oracle Fusion Cloud ERP: Incentive Compensation
// ═══════════════════════════════════════════════════════════════════

/// Sales Representative profile for commission tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesRepresentative {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rep_code: String,
    pub employee_id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub territory_code: Option<String>,
    pub territory_name: Option<String>,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
    pub hire_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Commission Plan defining how a rep earns commission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub basis: String,
    pub calculation_method: String,
    pub default_rate: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Commission rate tier within a plan (e.g., 0-10k at 5%, 10k-50k at 8%)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionRateTier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Uuid,
    pub tier_number: i32,
    pub from_amount: String,
    pub to_amount: Option<String>,
    pub rate_percent: String,
    pub flat_amount: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Plan assignment linking a rep to a commission plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rep_id: Uuid,
    pub plan_id: Uuid,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Sales Quota for a rep in a given period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesQuota {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rep_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub quota_number: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub quota_type: String,
    pub target_amount: String,
    pub achieved_amount: String,
    pub achievement_percent: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Commission transaction (a credited sale earning commission)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rep_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub quota_id: Option<Uuid>,
    pub transaction_number: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub sale_amount: String,
    pub commission_basis_amount: String,
    pub commission_rate: String,
    pub commission_amount: String,
    pub currency_code: String,
    pub status: String,
    pub payout_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Commission payout batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionPayout {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payout_number: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub total_payout_amount: String,
    pub currency_code: String,
    pub rep_count: i32,
    pub transaction_count: i32,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rejected_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Individual payout line per rep within a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionPayoutLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payout_id: Uuid,
    pub rep_id: Uuid,
    pub rep_name: String,
    pub plan_id: Option<Uuid>,
    pub plan_code: Option<String>,
    pub gross_commission: String,
    pub adjustment_amount: String,
    pub net_commission: String,
    pub currency_code: String,
    pub transaction_count: i32,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Commission Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionDashboardSummary {
    pub total_reps: i32,
    pub active_reps: i32,
    pub total_plans: i32,
    pub active_plans: i32,
    pub total_quotas: i32,
    pub total_transactions: i32,
    pub total_pending_payouts: i32,
    pub total_commission_this_month: String,
    pub total_quota_achievement_percent: String,
    pub payouts_by_status: serde_json::Value,
    pub top_performers: Vec<CommissionTopPerformer>,
}

/// Top performer in commission dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionTopPerformer {
    pub rep_id: Uuid,
    pub rep_name: String,
    pub total_commission: String,
    pub quota_achievement: String,
    pub rank: i32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Treasury Management (Oracle Fusion Treasury)
// ═══════════════════════════════════════════════════════════════════════════════

/// Counterparty (bank or financial institution) for treasury operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryCounterparty {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub counterparty_code: String,
    pub name: String,
    pub counterparty_type: String, // bank, financial_institution, internal
    pub country_code: Option<String>,
    pub credit_rating: Option<String>,
    pub credit_limit: Option<String>,
    pub settlement_currency: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Treasury deal (investment, borrowing, or FX deal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryDeal {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub deal_number: String,
    pub deal_type: String, // investment, borrowing, fx_forward, fx_spot
    pub description: Option<String>,
    pub counterparty_id: Uuid,
    pub counterparty_name: Option<String>,
    pub currency_code: String,
    pub principal_amount: String,
    pub interest_rate: Option<String>,
    pub interest_basis: Option<String>, // actual_360, actual_365, 30_360
    pub start_date: chrono::NaiveDate,
    pub maturity_date: chrono::NaiveDate,
    pub term_days: i32,
    /// For FX deals: the bought currency
    pub fx_buy_currency: Option<String>,
    /// For FX deals: the bought amount
    pub fx_buy_amount: Option<String>,
    /// For FX deals: the sold currency
    pub fx_sell_currency: Option<String>,
    /// For FX deals: the sold amount
    pub fx_sell_amount: Option<String>,
    /// For FX deals: the exchange rate
    pub fx_rate: Option<String>,
    pub accrued_interest: String,
    pub settlement_amount: Option<String>,
    pub gl_account_code: Option<String>,
    pub status: String, // draft, authorized, settled, matured, cancelled
    pub authorized_by: Option<Uuid>,
    pub authorized_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
    pub matured_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Treasury deal settlement record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasurySettlement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub deal_id: Uuid,
    pub settlement_number: String,
    pub settlement_type: String, // full, partial, early
    pub settlement_date: chrono::NaiveDate,
    pub principal_amount: String,
    pub interest_amount: String,
    pub total_amount: String,
    pub payment_reference: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub status: String, // pending, completed, reversed
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Treasury dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryDashboardSummary {
    pub total_active_deals: i32,
    pub total_investments: String,
    pub total_borrowings: String,
    pub total_fx_exposure: String,
    pub total_accrued_interest: String,
    pub deals_maturing_7_days: i32,
    pub deals_maturing_30_days: i32,
    pub investment_count: i32,
    pub borrowing_count: i32,
    pub fx_deal_count: i32,
    pub active_counterparties: i32,
    pub deals_by_status: serde_json::Value,
    pub deals_by_type: serde_json::Value,
    pub maturity_profile: serde_json::Value,
}

// ============================================================================
// Supplier Qualification Management (Oracle Fusion Procurement > Supplier Qualification)
// ============================================================================

/// Qualification Area - defines a category of qualification criteria.
/// Oracle Fusion: Procurement > Supplier Qualification > Areas
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationArea {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub area_code: String,
    pub name: String,
    pub description: Option<String>,
    /// "questionnaire", "certificate", "financial", "`site_visit`", "reference", "other"
    pub area_type: String,
    /// "manual", "weighted", "`pass_fail`"
    pub scoring_model: String,
    pub passing_score: String,
    pub is_mandatory: bool,
    pub renewal_period_days: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Qualification Question - individual question within an area.
/// Oracle Fusion: Procurement > Supplier Qualification > Questions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationQuestion {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub area_id: Uuid,
    pub question_number: i32,
    pub question_text: String,
    pub description: Option<String>,
    /// "text", "`yes_no`", "numeric", "date", "`multi_choice`", "`file_upload`"
    pub response_type: String,
    pub choices: Option<serde_json::Value>,
    pub is_required: bool,
    pub weight: String,
    pub max_score: String,
    pub help_text: Option<String>,
    pub display_order: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Qualification Initiative - a qualification campaign/run.
/// Oracle Fusion: Procurement > Supplier Qualification > Initiatives
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierQualificationInitiative {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub initiative_number: String,
    pub name: String,
    pub description: Option<String>,
    pub area_id: Uuid,
    /// "`new_supplier`", "requalification", "compliance", "`ad_hoc`"
    pub qualification_purpose: String,
    /// "draft", "active", "`pending_evaluations`", "completed", "cancelled"
    pub status: String,
    pub deadline: Option<chrono::NaiveDate>,
    pub total_invited: i32,
    pub total_responded: i32,
    pub total_qualified: i32,
    pub total_disqualified: i32,
    pub total_pending: i32,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Qualification Invitation - per-supplier qualification within an initiative.
/// Oracle Fusion: Procurement > Supplier Qualification > Invitations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierQualificationInvitation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub initiative_id: Uuid,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub supplier_contact_name: Option<String>,
    pub supplier_contact_email: Option<String>,
    /// "initiated", "`pending_response`", "`under_evaluation`", "qualified", "disqualified", "expired", "withdrawn"
    pub status: String,
    pub invitation_date: Option<DateTime<Utc>>,
    pub response_date: Option<DateTime<Utc>>,
    pub evaluation_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub overall_score: String,
    pub max_possible_score: String,
    pub score_percentage: String,
    pub qualified_by: Option<Uuid>,
    pub disqualified_reason: Option<String>,
    pub evaluation_notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Qualification Response - individual answer to a question.
/// Oracle Fusion: Procurement > Supplier Qualification > Responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierQualificationResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invitation_id: Uuid,
    pub question_id: Uuid,
    pub response_text: Option<String>,
    pub response_value: Option<serde_json::Value>,
    pub file_reference: Option<String>,
    pub score: String,
    pub max_score: String,
    pub evaluator_notes: Option<String>,
    pub evaluated_by: Option<Uuid>,
    pub evaluated_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Certification - track ongoing supplier certifications.
/// Oracle Fusion: Procurement > Supplier Qualification > Certifications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierCertification {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub certification_type: String,
    pub certification_name: String,
    pub certifying_body: Option<String>,
    pub certificate_number: Option<String>,
    /// "active", "expired", "revoked", "`pending_renewal`"
    pub status: String,
    pub issued_date: Option<chrono::NaiveDate>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub renewal_date: Option<chrono::NaiveDate>,
    pub qualification_invitation_id: Option<Uuid>,
    pub document_reference: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Qualification Dashboard Summary.
/// Oracle Fusion: Procurement > Supplier Qualification > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierQualificationDashboardSummary {
    pub total_active_areas: i32,
    pub total_active_initiatives: i32,
    pub total_suppliers_invited: i32,
    pub total_suppliers_qualified: i32,
    pub total_suppliers_pending: i32,
    pub total_suppliers_disqualified: i32,
    pub total_certifications_active: i32,
    pub total_certifications_expiring_30_days: i32,
    pub qualification_rate_percent: String,
    pub initiatives_by_status: serde_json::Value,
    pub certifications_by_type: serde_json::Value,
}

// ============================================================================
// Purchase Requisitions (Oracle Fusion Self-Service Procurement > Requisitions)
// ============================================================================

/// Purchase Requisition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRequisition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_number: String,
    pub description: Option<String>,
    pub urgency_code: String,
    pub status: String,
    pub requester_id: Option<Uuid>,
    pub requester_name: Option<String>,
    pub department: Option<String>,
    pub justification: Option<String>,
    pub budget_code: Option<String>,
    pub amount_limit: Option<String>,
    pub total_amount: String,
    pub currency_code: String,
    pub charge_account_code: Option<String>,
    pub delivery_address: Option<String>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub lines: Vec<RequisitionLine>,
    pub approvals: Vec<RequisitionApproval>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Requisition Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_id: Uuid,
    pub line_number: i32,
    pub item_code: Option<String>,
    pub item_description: String,
    pub category: Option<String>,
    pub quantity: String,
    pub unit_of_measure: String,
    pub unit_price: String,
    pub line_amount: String,
    pub currency_code: String,
    pub charge_account_code: Option<String>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub status: String,
    pub source_type: String,
    pub source_reference: Option<String>,
    pub notes: Option<String>,
    pub distributions: Vec<RequisitionDistribution>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Requisition Distribution (accounting split)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionDistribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_id: Uuid,
    pub line_id: Uuid,
    pub distribution_number: i32,
    pub charge_account_code: String,
    pub allocation_percentage: String,
    pub amount: String,
    pub project_code: Option<String>,
    pub cost_center: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Requisition Approval
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionApproval {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_id: Uuid,
    pub approver_id: Uuid,
    pub approver_name: Option<String>,
    pub action: String,
    pub comments: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// `AutoCreate` Link (requisition to PO tracking)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutocreateLink {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_id: Uuid,
    pub requisition_line_id: Uuid,
    pub purchase_order_id: Option<Uuid>,
    pub purchase_order_number: Option<String>,
    pub purchase_order_line_id: Option<Uuid>,
    pub purchase_order_line_number: Option<i32>,
    pub quantity_ordered: String,
    pub status: String,
    pub autocreate_date: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Purchase Requisition Create Request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRequisitionRequest {
    pub description: Option<String>,
    pub urgency_code: Option<String>,
    pub requester_id: Option<Uuid>,
    pub requester_name: Option<String>,
    pub department: Option<String>,
    pub justification: Option<String>,
    pub budget_code: Option<String>,
    pub amount_limit: Option<String>,
    pub currency_code: Option<String>,
    pub charge_account_code: Option<String>,
    pub delivery_address: Option<String>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
    pub lines: Vec<RequisitionLineRequest>,
}

/// Requisition Line Create Request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionLineRequest {
    pub item_code: Option<String>,
    pub item_description: String,
    pub category: Option<String>,
    pub quantity: Option<String>,
    pub unit_of_measure: Option<String>,
    pub unit_price: Option<String>,
    pub currency_code: Option<String>,
    pub charge_account_code: Option<String>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub source_type: Option<String>,
    pub source_reference: Option<String>,
    pub notes: Option<String>,
    pub distributions: Option<Vec<RequisitionDistributionRequest>>,
}

/// Requisition Distribution Create Request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionDistributionRequest {
    pub charge_account_code: String,
    pub allocation_percentage: Option<String>,
    pub amount: Option<String>,
    pub project_code: Option<String>,
    pub cost_center: Option<String>,
}

/// `AutoCreate` Request (convert requisitions to POs)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutocreateRequest {
    pub requisition_line_ids: Vec<Uuid>,
    pub purchase_order_number: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
}

/// Requisition Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionDashboardSummary {
    pub total_requisitions: i32,
    pub draft_requisitions: i32,
    pub submitted_requisitions: i32,
    pub approved_requisitions: i32,
    pub rejected_requisitions: i32,
    pub cancelled_requisitions: i32,
    pub total_amount: String,
    pub autocreate_pending: i32,
    pub autocreate_ordered: i32,
    pub by_priority: serde_json::Value,
}

// ============================================================================
// Product Information Management (PIM)
// Oracle Fusion Cloud: Product Hub / Product Information Management
//
// Central product master data management including items, categories,
// cross-references, lifecycle phases, and new item request workflows.
// ============================================================================

/// Item status within its lifecycle.
/// Oracle Fusion: Product Development > Item > Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    Draft,
    Active,
    Obsolete,
    Inactive,
    PendingApproval,
}

impl std::fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Draft => write!(f, "draft"),
            Self::Active => write!(f, "active"),
            Self::Obsolete => write!(f, "obsolete"),
            Self::Inactive => write!(f, "inactive"),
            Self::PendingApproval => write!(f, "pending_approval"),
        }
    }
}

/// Lifecycle phase for an item.
/// Oracle Fusion: Product Hub > Item Lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecyclePhase {
    Concept,
    Design,
    Prototype,
    Production,
    PhaseOut,
    Obsolete,
}

impl std::fmt::Display for LifecyclePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Concept => write!(f, "concept"),
            Self::Design => write!(f, "design"),
            Self::Prototype => write!(f, "prototype"),
            Self::Production => write!(f, "production"),
            Self::PhaseOut => write!(f, "phase_out"),
            Self::Obsolete => write!(f, "obsolete"),
        }
    }
}

/// Product item master record.
/// Oracle Fusion: Product Hub > Manage Items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_number: String,
    pub item_name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub item_type: String, // finished_good, subassembly, component, service, supply, expense
    pub status: String,
    pub lifecycle_phase: String,
    pub primary_uom_code: String,
    pub secondary_uom_code: Option<String>,
    pub weight: Option<String>,
    pub weight_uom: Option<String>,
    pub volume: Option<String>,
    pub volume_uom: Option<String>,
    pub hazmat_flag: bool,
    pub lot_control_flag: bool,
    pub serial_control_flag: bool,
    pub shelf_life_days: Option<i32>,
    pub min_order_quantity: Option<String>,
    pub max_order_quantity: Option<String>,
    pub lead_time_days: Option<i32>,
    pub list_price: Option<String>,
    pub cost_price: Option<String>,
    pub currency_code: String,
    pub inventory_item_flag: bool,
    pub purchasable_flag: bool,
    pub sellable_flag: bool,
    pub stock_enabled_flag: bool,
    pub invoice_enabled_flag: bool,
    pub default_buyer_id: Option<Uuid>,
    pub default_supplier_id: Option<Uuid>,
    pub template_id: Option<Uuid>,
    pub thumbnail_url: Option<String>,
    pub image_url: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// PIM Item category for hierarchical classification.
/// Oracle Fusion: Product Hub > Manage Item Classes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub level_number: i32,
    pub item_count: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// PIM item-to-category assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimCategoryAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_id: Uuid,
    pub category_id: Uuid,
    pub is_primary: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Item cross-reference for mapping item identifiers across systems.
/// Oracle Fusion: Product Hub > Manage Cross-References
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimCrossReference {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_id: Uuid,
    pub cross_reference_type: String, // gtin, upc, ean, supplier, customer, internal, other
    pub cross_reference_value: String,
    pub description: Option<String>,
    pub source_system: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New Item Request (NIR) for introducing new products through workflow.
/// Oracle Fusion: Product Hub > New Item Request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimNewItemRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_number: String,
    pub title: String,
    pub description: Option<String>,
    pub item_type: String,
    pub priority: String, // low, medium, high, critical
    pub status: String,   // draft, submitted, approved, rejected, implemented, cancelled
    pub requested_item_number: Option<String>,
    pub requested_item_name: Option<String>,
    pub requested_category_id: Option<Uuid>,
    pub justification: Option<String>,
    pub target_launch_date: Option<chrono::NaiveDate>,
    pub estimated_cost: Option<String>,
    pub currency_code: String,
    pub requested_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub implemented_item_id: Option<Uuid>,
    pub implemented_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Item template for standardizing item creation.
/// Oracle Fusion: Product Hub > Manage Item Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimItemTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub item_type: String,
    pub default_uom_code: Option<String>,
    pub default_category_id: Option<Uuid>,
    pub default_inventory_flag: bool,
    pub default_purchasable_flag: bool,
    pub default_sellable_flag: bool,
    pub default_stock_enabled_flag: bool,
    pub attribute_defaults: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// PIM Dashboard summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimDashboard {
    pub total_items: i32,
    pub active_items: i32,
    pub draft_items: i32,
    pub obsolete_items: i32,
    pub total_categories: i32,
    pub pending_nir_count: i32,
    pub approved_nir_count: i32,
    pub cross_reference_count: i32,
    pub recently_created_items: i32,
    pub items_by_type: serde_json::Value,
}

// ============================================================================
// Order Management (Oracle Fusion SCM > Order Management)
// ============================================================================

/// Sales Order Header
/// Oracle Fusion equivalent: Order Management > Sales Orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrder {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub order_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub customer_po_number: Option<String>,
    pub order_date: chrono::NaiveDate,
    pub requested_ship_date: Option<chrono::NaiveDate>,
    pub actual_ship_date: Option<chrono::NaiveDate>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub actual_delivery_date: Option<chrono::NaiveDate>,
    pub ship_to_address: Option<String>,
    pub bill_to_address: Option<String>,
    pub currency_code: String,
    pub subtotal_amount: String,
    pub tax_amount: String,
    pub shipping_charges: String,
    pub total_amount: String,
    pub payment_terms: Option<String>,
    pub shipping_method: Option<String>,
    pub sales_channel: Option<String>,
    pub salesperson_id: Option<Uuid>,
    pub salesperson_name: Option<String>,
    pub status: String,
    pub fulfillment_status: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sales Order Line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrderLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub order_id: Uuid,
    pub line_number: i32,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity_ordered: String,
    pub quantity_shipped: String,
    pub quantity_cancelled: String,
    pub quantity_backordered: String,
    pub unit_selling_price: String,
    pub unit_list_price: Option<String>,
    pub line_amount: String,
    pub discount_percent: Option<String>,
    pub discount_amount: Option<String>,
    pub tax_code: Option<String>,
    pub tax_amount: String,
    pub requested_ship_date: Option<chrono::NaiveDate>,
    pub actual_ship_date: Option<chrono::NaiveDate>,
    pub promised_delivery_date: Option<chrono::NaiveDate>,
    pub ship_from_warehouse: Option<String>,
    pub fulfillment_status: String,
    pub status: String,
    pub cancellation_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Order Hold
/// Oracle Fusion: Order Management > Order Holds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderHold {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub order_id: Uuid,
    pub order_line_id: Option<Uuid>,
    pub hold_type: String,
    pub hold_reason: String,
    pub applied_by: Option<Uuid>,
    pub applied_by_name: Option<String>,
    pub released_by: Option<Uuid>,
    pub released_by_name: Option<String>,
    pub released_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fulfillment Shipment
/// Oracle Fusion: Order Management > Shipments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentShipment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_number: String,
    pub order_id: Uuid,
    pub order_line_ids: serde_json::Value,
    pub warehouse: Option<String>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub shipping_method: Option<String>,
    pub ship_date: Option<chrono::NaiveDate>,
    pub estimated_delivery_date: Option<chrono::NaiveDate>,
    pub actual_delivery_date: Option<chrono::NaiveDate>,
    pub delivery_confirmation: Option<String>,
    pub status: String,
    pub shipped_by: Option<Uuid>,
    pub shipped_by_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create Sales Order Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSalesOrderRequest {
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub customer_po_number: Option<String>,
    pub order_date: chrono::NaiveDate,
    pub requested_ship_date: Option<chrono::NaiveDate>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub ship_to_address: Option<String>,
    pub bill_to_address: Option<String>,
    pub currency_code: String,
    pub payment_terms: Option<String>,
    pub shipping_method: Option<String>,
    pub sales_channel: Option<String>,
    pub salesperson_id: Option<Uuid>,
    pub salesperson_name: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Add Order Line Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddOrderLineRequest {
    pub org_id: Uuid,
    pub order_id: Uuid,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity_ordered: String,
    pub unit_selling_price: String,
    pub unit_list_price: Option<String>,
    pub discount_percent: Option<String>,
    pub discount_amount: Option<String>,
    pub tax_code: Option<String>,
    pub requested_ship_date: Option<chrono::NaiveDate>,
    pub promised_delivery_date: Option<chrono::NaiveDate>,
    pub ship_from_warehouse: Option<String>,
}

/// Order Management Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderManagementDashboard {
    pub total_orders: i32,
    pub open_orders: i32,
    pub orders_in_fulfillment: i32,
    pub completed_orders: i32,
    pub cancelled_orders: i32,
    pub total_order_value: String,
    pub average_order_value: String,
    pub orders_on_hold: i32,
    pub backordered_lines: i32,
    pub overdue_shipments: i32,
    pub orders_by_status: serde_json::Value,
    pub orders_by_channel: serde_json::Value,
    pub fulfillment_rate_pct: String,
    pub on_time_shipment_pct: String,
}

// ============================================================================
// Demand Planning / Demand Management (Oracle Fusion SCM > Demand Management)
// ============================================================================

/// Forecast method definition
/// Oracle Fusion: Demand Management > Forecast Methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandForecastMethod {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub method_type: String,
    pub parameters: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Demand schedule (forecast header)
/// Oracle Fusion: Demand Management > Demand Schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_number: String,
    pub name: String,
    pub description: Option<String>,
    pub method_id: Option<Uuid>,
    pub method_name: Option<String>,
    pub schedule_type: String,
    pub status: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub currency_code: String,
    pub total_forecast_quantity: String,
    pub total_forecast_value: String,
    pub confidence_level: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Demand schedule line (forecast item per period)
/// Oracle Fusion: Demand Management > Schedule Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandScheduleLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_id: Uuid,
    pub line_number: i32,
    pub item_code: String,
    pub item_name: Option<String>,
    pub item_category: Option<String>,
    pub warehouse_code: Option<String>,
    pub region: Option<String>,
    pub customer_group: Option<String>,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub forecast_quantity: String,
    pub forecast_value: String,
    pub unit_price: String,
    pub consumed_quantity: String,
    pub remaining_quantity: String,
    pub confidence_pct: String,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Historical demand data (actuals)
/// Oracle Fusion: Demand Management > Demand History
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandHistory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_code: String,
    pub item_name: Option<String>,
    pub warehouse_code: Option<String>,
    pub region: Option<String>,
    pub customer_group: Option<String>,
    pub actual_date: chrono::NaiveDate,
    pub actual_quantity: String,
    pub actual_value: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_line_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Forecast consumption entry
/// Oracle Fusion: Demand Management > Forecast Consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandConsumption {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_line_id: Uuid,
    pub history_id: Option<Uuid>,
    pub consumed_quantity: String,
    pub consumed_date: chrono::NaiveDate,
    pub source_type: String,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Forecast accuracy measurement
/// Oracle Fusion: Demand Management > Accuracy Analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandAccuracy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_id: Uuid,
    pub schedule_line_id: Option<Uuid>,
    pub item_code: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub forecast_quantity: String,
    pub actual_quantity: String,
    pub absolute_error: String,
    pub absolute_pct_error: String,
    pub bias: String,
    pub measurement_date: chrono::NaiveDate,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Demand Planning Dashboard
/// Oracle Fusion: Demand Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandPlanningDashboard {
    pub total_schedules: i32,
    pub active_schedules: i32,
    pub total_forecast_items: i32,
    pub total_forecast_quantity: String,
    pub total_forecast_value: String,
    pub avg_accuracy_pct: String,
    pub schedules_by_status: serde_json::Value,
    pub top_forecast_items: serde_json::Value,
    pub accuracy_by_method: serde_json::Value,
}

// ============================================================================
// Shipping Execution (Oracle Fusion SCM > Shipping Execution)
// ============================================================================

/// Shipping Carrier
/// Oracle Fusion: SCM > Shipping > Carriers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShippingCarrier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub carrier_type: String,
    pub tracking_url_template: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Shipping Method
/// Oracle Fusion: SCM > Shipping > Shipping Methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShippingMethod {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub carrier_id: Option<Uuid>,
    pub transit_time_days: i32,
    pub is_express: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Shipment (shipping header)
/// Oracle Fusion: SCM > Shipping > Shipments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shipment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_number: String,
    pub description: Option<String>,
    pub status: String,
    pub carrier_id: Option<Uuid>,
    pub carrier_name: Option<String>,
    pub shipping_method_id: Option<Uuid>,
    pub shipping_method_name: Option<String>,
    pub order_id: Option<Uuid>,
    pub order_number: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub ship_from_warehouse: Option<String>,
    pub ship_to_name: Option<String>,
    pub ship_to_address: Option<String>,
    pub ship_to_city: Option<String>,
    pub ship_to_state: Option<String>,
    pub ship_to_postal_code: Option<String>,
    pub ship_to_country: Option<String>,
    pub tracking_number: Option<String>,
    pub total_weight: String,
    pub weight_unit: String,
    pub total_volume: String,
    pub volume_unit: String,
    pub total_packages: i32,
    pub shipped_date: Option<DateTime<Utc>>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
    pub actual_delivery: Option<DateTime<Utc>>,
    pub confirmed_by: Option<Uuid>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub shipped_by: Option<Uuid>,
    pub delivered_by: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Shipment Line
/// Oracle Fusion: SCM > Shipping > Shipment Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_id: Uuid,
    pub line_number: i32,
    pub order_line_id: Option<Uuid>,
    pub item_code: String,
    pub item_name: Option<String>,
    pub item_description: Option<String>,
    pub requested_quantity: String,
    pub shipped_quantity: String,
    pub backordered_quantity: String,
    pub unit_of_measure: String,
    pub weight: String,
    pub weight_unit: String,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub is_fragile: bool,
    pub is_hazardous: bool,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Packing Slip
/// Oracle Fusion: SCM > Shipping > Packing Slips
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackingSlip {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_id: Uuid,
    pub packing_slip_number: String,
    pub package_number: i32,
    pub package_type: String,
    pub weight: String,
    pub weight_unit: String,
    pub dimensions_length: String,
    pub dimensions_width: String,
    pub dimensions_height: String,
    pub dimensions_unit: String,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Packing Slip Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackingSlipLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub packing_slip_id: Uuid,
    pub shipment_line_id: Uuid,
    pub line_number: i32,
    pub item_code: String,
    pub item_name: Option<String>,
    pub packed_quantity: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Shipping Execution Dashboard
/// Oracle Fusion: SCM > Shipping > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShippingDashboard {
    pub total_shipments: i32,
    pub pending_shipments: i32,
    pub shipped_this_month: i32,
    pub delivered_this_month: i32,
    pub total_carriers: i32,
    pub shipments_by_status: serde_json::Value,
    pub recent_shipments: serde_json::Value,
    pub top_carriers: serde_json::Value,
}

// ============================================================================
// Product Configurator (Oracle Fusion Cloud SCM > Product Management > Configurator)
// ============================================================================

/// Configuration Model - defines a configurable product structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigModel {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub model_number: String,
    pub name: String,
    pub description: Option<String>,
    pub base_product_id: Option<Uuid>,
    pub base_product_number: Option<String>,
    pub base_product_name: Option<String>,
    pub model_type: String,
    pub status: String,
    pub version: i32,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub default_config: serde_json::Value,
    pub validation_mode: String,
    pub ui_layout: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Configuration Feature - a group of choices (e.g. "Color", "Engine Type")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFeature {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub model_id: Uuid,
    pub feature_code: String,
    pub name: String,
    pub description: Option<String>,
    pub feature_type: String,
    pub is_required: bool,
    pub display_order: i32,
    pub ui_hints: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Configuration Option - a specific choice within a feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOption {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub feature_id: Uuid,
    pub option_code: String,
    pub name: String,
    pub description: Option<String>,
    pub option_type: String,
    pub price_adjustment: f64,
    pub cost_adjustment: f64,
    pub lead_time_days: i32,
    pub is_default: bool,
    pub is_available: bool,
    pub display_order: i32,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Configuration Rule - constraints between features/options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub model_id: Uuid,
    pub rule_code: String,
    pub name: String,
    pub description: Option<String>,
    pub rule_type: String,
    pub source_feature_id: Option<Uuid>,
    pub source_option_id: Option<Uuid>,
    pub target_feature_id: Option<Uuid>,
    pub target_option_id: Option<Uuid>,
    pub condition_expression: Option<String>,
    pub severity: String,
    pub is_active: bool,
    pub priority: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Configuration Instance - an actual configuration created from a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInstance {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub instance_number: String,
    pub model_id: Uuid,
    pub model_number: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub selections: serde_json::Value,
    pub validation_errors: serde_json::Value,
    pub validation_warnings: serde_json::Value,
    pub base_price: f64,
    pub total_price: f64,
    pub currency_code: String,
    pub config_hash: Option<String>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub valid_from: Option<chrono::DateTime<chrono::Utc>>,
    pub valid_to: Option<chrono::DateTime<chrono::Utc>>,
    pub sales_order_id: Option<Uuid>,
    pub sales_order_number: Option<String>,
    pub sales_order_line: Option<i32>,
    pub configured_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Product Configurator Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguratorDashboard {
    pub total_models: i32,
    pub active_models: i32,
    pub total_configurations: i32,
    pub valid_configurations: i32,
    pub invalid_configurations: i32,
    pub ordered_configurations: i32,
    pub total_rules: i32,
    pub active_rules: i32,
    pub avg_configuration_price: f64,
    pub total_configured_value: f64,
    pub models_by_status: serde_json::Value,
    pub configurations_by_status: serde_json::Value,
    pub top_configured_models: serde_json::Value,
}

// ============================================================================
// Transportation Management (Oracle Fusion Cloud SCM > Transportation Management)
// ============================================================================

/// Carrier - a shipping partner (`FedEx`, UPS, DHL, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Carrier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub carrier_code: String,
    pub name: String,
    pub description: Option<String>,
    pub carrier_type: String,
    pub status: String,
    pub scac_code: Option<String>,
    pub dot_number: Option<String>,
    pub mc_number: Option<String>,
    pub tax_id: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: String,
    pub currency_code: String,
    pub payment_terms: String,
    pub insurance_policy_number: Option<String>,
    pub insurance_expiry_date: Option<chrono::NaiveDate>,
    pub performance_rating: f64,
    pub on_time_delivery_pct: f64,
    pub claims_ratio: f64,
    pub default_service_level: String,
    pub capabilities: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Carrier Service - a specific service level offered by a carrier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarrierService {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub carrier_id: Uuid,
    pub service_code: String,
    pub name: String,
    pub description: Option<String>,
    pub service_level: String,
    pub transit_days_min: i32,
    pub transit_days_max: i32,
    pub max_weight_kg: Option<f64>,
    pub max_dimensions: Option<serde_json::Value>,
    pub cutoff_time: Option<chrono::NaiveTime>,
    pub operates_on_weekends: bool,
    pub is_international: bool,
    pub rate_per_kg: f64,
    pub minimum_charge: f64,
    pub fuel_surcharge_pct: f64,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Transport Lane - an origin-destination route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportLane {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub lane_code: String,
    pub name: String,
    pub description: Option<String>,
    pub origin_location_id: Option<Uuid>,
    pub origin_location_name: Option<String>,
    pub origin_city: Option<String>,
    pub origin_state: Option<String>,
    pub origin_country: String,
    pub origin_postal_code: Option<String>,
    pub destination_location_id: Option<Uuid>,
    pub destination_location_name: Option<String>,
    pub destination_city: Option<String>,
    pub destination_state: Option<String>,
    pub destination_country: String,
    pub destination_postal_code: Option<String>,
    pub distance_km: Option<f64>,
    pub estimated_transit_hours: Option<f64>,
    pub lane_type: String,
    pub preferred_carrier_id: Option<Uuid>,
    pub preferred_service_id: Option<Uuid>,
    pub status: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub restrictions: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Transport Shipment - a master shipment for transporting goods (Transportation Management)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportShipment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_number: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub shipment_type: String,
    pub priority: String,
    pub carrier_id: Option<Uuid>,
    pub carrier_code: Option<String>,
    pub carrier_name: Option<String>,
    pub carrier_service_id: Option<Uuid>,
    pub carrier_service_code: Option<String>,
    pub lane_id: Option<Uuid>,
    pub lane_code: Option<String>,
    pub origin_location_id: Option<Uuid>,
    pub origin_location_name: Option<String>,
    pub origin_address: serde_json::Value,
    pub destination_location_id: Option<Uuid>,
    pub destination_location_name: Option<String>,
    pub destination_address: serde_json::Value,
    pub planned_ship_date: Option<chrono::NaiveDate>,
    pub actual_ship_date: Option<chrono::NaiveDate>,
    pub planned_delivery_date: Option<chrono::NaiveDate>,
    pub actual_delivery_date: Option<chrono::NaiveDate>,
    pub pickup_window_start: Option<chrono::DateTime<chrono::Utc>>,
    pub pickup_window_end: Option<chrono::DateTime<chrono::Utc>>,
    pub delivery_window_start: Option<chrono::DateTime<chrono::Utc>>,
    pub delivery_window_end: Option<chrono::DateTime<chrono::Utc>>,
    pub total_weight_kg: f64,
    pub total_volume_cbm: f64,
    pub total_pieces: i32,
    pub freight_cost: f64,
    pub fuel_surcharge: f64,
    pub accessorial_charges: f64,
    pub total_cost: f64,
    pub currency_code: String,
    pub tracking_number: Option<String>,
    pub tracking_url: Option<String>,
    pub pro_number: Option<String>,
    pub bill_of_lading: Option<String>,
    pub sales_order_id: Option<Uuid>,
    pub sales_order_number: Option<String>,
    pub purchase_order_id: Option<Uuid>,
    pub purchase_order_number: Option<String>,
    pub transfer_order_id: Option<Uuid>,
    pub special_instructions: Option<String>,
    pub declared_value: Option<f64>,
    pub insurance_required: bool,
    pub signature_required: bool,
    pub temperature_requirements: Option<serde_json::Value>,
    pub hazmat_info: Option<serde_json::Value>,
    pub driver_name: Option<String>,
    pub vehicle_id: Option<String>,
    pub metadata: serde_json::Value,
    pub booked_by: Option<Uuid>,
    pub shipped_by: Option<Uuid>,
    pub received_by: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Transport Shipment Stop - a pickup/delivery stop on a multi-stop route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportShipmentStop {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_id: Uuid,
    pub stop_number: i32,
    pub stop_type: String,
    pub location_id: Option<Uuid>,
    pub location_name: Option<String>,
    pub address: serde_json::Value,
    pub planned_arrival: Option<chrono::DateTime<chrono::Utc>>,
    pub actual_arrival: Option<chrono::DateTime<chrono::Utc>>,
    pub planned_departure: Option<chrono::DateTime<chrono::Utc>>,
    pub actual_departure: Option<chrono::DateTime<chrono::Utc>>,
    pub status: String,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub special_instructions: Option<String>,
    pub pieces: i32,
    pub weight_kg: f64,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Transport Shipment Line - an item within a transport shipment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportShipmentLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_id: Uuid,
    pub line_number: i32,
    pub item_id: Option<Uuid>,
    pub item_number: Option<String>,
    pub item_description: Option<String>,
    pub quantity: i32,
    pub quantity_shipped: i32,
    pub quantity_received: i32,
    pub unit_of_measure: String,
    pub weight_kg: f64,
    pub volume_cbm: f64,
    pub lot_number: Option<String>,
    pub serial_numbers: serde_json::Value,
    pub source_line_id: Option<Uuid>,
    pub source_line_type: Option<String>,
    pub stop_id: Option<Uuid>,
    pub freight_class: Option<String>,
    pub nmfc_code: Option<String>,
    pub hazmat_class: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Transport Shipment Tracking Event - a status update during transit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportShipmentTrackingEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_id: Uuid,
    pub event_type: String,
    pub event_timestamp: chrono::DateTime<chrono::Utc>,
    pub location_description: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub description: Option<String>,
    pub carrier_event_code: Option<String>,
    pub carrier_event_description: Option<String>,
    pub updated_by: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Freight Rate - a carrier rate agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreightRate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rate_code: String,
    pub name: String,
    pub description: Option<String>,
    pub carrier_id: Uuid,
    pub carrier_service_id: Option<Uuid>,
    pub lane_id: Option<Uuid>,
    pub rate_type: String,
    pub rate_amount: f64,
    pub minimum_charge: f64,
    pub currency_code: String,
    pub fuel_surcharge_pct: f64,
    pub accessorial_rates: serde_json::Value,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub is_contract_rate: bool,
    pub contract_number: Option<String>,
    pub volume_threshold_min: Option<f64>,
    pub volume_threshold_max: Option<f64>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Transportation Management Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportationDashboard {
    pub total_shipments: i32,
    pub active_shipments: i32,
    pub delivered_shipments: i32,
    pub delayed_shipments: i32,
    pub exception_shipments: i32,
    pub total_carriers: i32,
    pub active_carriers: i32,
    pub total_lanes: i32,
    pub active_lanes: i32,
    pub on_time_delivery_pct: f64,
    pub avg_transit_days: f64,
    pub total_freight_cost: f64,
    pub avg_cost_per_kg: f64,
    pub shipments_by_status: serde_json::Value,
    pub shipments_by_carrier: serde_json::Value,
    pub cost_by_carrier: serde_json::Value,
    pub top_lanes: serde_json::Value,
}

// ============================================================================
// Cost Accounting (Oracle Fusion Cost Management)
// ============================================================================

/// Cost Book - defines a costing method for an organization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostBook {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub costing_method: String, // standard, average, fifo, lifo
    pub currency_code: String,
    pub is_active: bool,
    pub status: String, // active, inactive
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Cost Element - material, labor, overhead, subcontracting, expense
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostElement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub element_type: String, // material, labor, overhead, subcontracting, expense
    pub cost_book_id: Option<Uuid>,
    pub is_active: bool,
    pub default_rate: String,
    pub rate_uom: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Cost Profile - item-level costing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostProfile {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub cost_book_id: Uuid,
    pub item_id: Option<Uuid>,
    pub item_name: Option<String>,
    pub cost_type: String, // standard, average, fifo, lifo
    pub lot_level_costing: bool,
    pub include_landed_costs: bool,
    pub overhead_absorption_method: String, // rate, amount, percentage
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Standard Cost per item per cost book per cost element
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardCost {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cost_book_id: Uuid,
    pub cost_profile_id: Option<Uuid>,
    pub cost_element_id: Uuid,
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub standard_cost: String,
    pub currency_code: String,
    pub effective_date: chrono::NaiveDate,
    pub status: String, // active, pending, superseded
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Cost Adjustment header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostAdjustment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub adjustment_number: String,
    pub cost_book_id: Uuid,
    pub adjustment_type: String, // standard_cost_update, cost_correction, revaluation, overhead_adjustment
    pub description: Option<String>,
    pub reason: Option<String>,
    pub status: String, // draft, submitted, approved, rejected, posted
    pub total_adjustment_amount: String,
    pub currency_code: String,
    pub effective_date: Option<chrono::NaiveDate>,
    pub posted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub posted_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub rejected_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Cost Adjustment Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostAdjustmentLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub adjustment_id: Uuid,
    pub line_number: i32,
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub cost_element_id: Option<Uuid>,
    pub old_cost: String,
    pub new_cost: String,
    pub adjustment_amount: String,
    pub currency_code: String,
    pub effective_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Cost Variance entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostVariance {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cost_book_id: Uuid,
    pub variance_type: String, // purchase_price, routing, overhead, rate, usage, mix
    pub variance_date: chrono::NaiveDate,
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub cost_element_id: Option<Uuid>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub standard_cost: String,
    pub actual_cost: String,
    pub variance_amount: String,
    pub variance_percent: String,
    pub quantity: String,
    pub currency_code: String,
    pub accounting_period: Option<String>,
    pub is_analyzed: bool,
    pub analysis_notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ============================================================================
// Supply Chain Planning (MRP) Types
// Oracle Fusion equivalent: Supply Chain Management > Supply Chain Planning
// ============================================================================

/// Planning Scenario - defines the context for a planning run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningScenario {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scenario_number: String,
    pub name: String,
    pub description: Option<String>,
    pub scenario_type: String,
    pub status: String,
    pub planning_horizon_days: i32,
    pub planning_start_date: Option<chrono::NaiveDate>,
    pub planning_end_date: Option<chrono::NaiveDate>,
    pub include_existing_supply: bool,
    pub include_on_hand: bool,
    pub include_work_in_progress: bool,
    pub auto_firm: bool,
    pub auto_firm_days: Option<i32>,
    pub net_shortages_only: bool,
    pub total_planned_orders: i32,
    pub total_exceptions: i32,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Planning Parameters - item-level planning attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningParameter {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub item_number: Option<String>,
    pub planner_code: Option<String>,
    pub planning_method: String,
    pub make_buy: String,
    pub lead_time_days: i32,
    pub safety_stock_quantity: String,
    pub min_order_quantity: String,
    pub max_order_quantity: Option<String>,
    pub fixed_order_quantity: Option<String>,
    pub fixed_lot_multiplier: String,
    pub order_multiple: String,
    pub planning_time_fence_days: i32,
    pub release_time_fence_days: i32,
    pub shrinkage_rate: String,
    pub lot_size_policy: String,
    pub period_order_quantity_days: Option<i32>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_name: Option<String>,
    pub default_supplier_id: Option<Uuid>,
    pub default_supplier_name: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Supply/Demand Entry - netting input for planning
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplyDemandEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scenario_id: Option<Uuid>,
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub item_number: Option<String>,
    pub entry_type: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub quantity: String,
    pub quantity_remaining: String,
    pub due_date: chrono::NaiveDate,
    pub priority: i32,
    pub status: String,
    pub pegged_to_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Planned Order - output of MRP planning run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedOrder {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scenario_id: Option<Uuid>,
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub item_number: Option<String>,
    pub order_number: String,
    pub order_type: String,
    pub status: String,
    pub quantity: String,
    pub quantity_firmed: String,
    pub due_date: chrono::NaiveDate,
    pub start_date: Option<chrono::NaiveDate>,
    pub need_date: Option<chrono::NaiveDate>,
    pub planner_notes: Option<String>,
    pub planning_priority: i32,
    pub order_action: String,
    pub suggested_supplier_id: Option<Uuid>,
    pub suggested_supplier_name: Option<String>,
    pub suggested_source_type: Option<String>,
    pub suggested_source_id: Option<Uuid>,
    pub firm_deadline: Option<chrono::NaiveDate>,
    pub pegging_demand_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Planning Exception - issues flagged during planning run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningException {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scenario_id: Option<Uuid>,
    pub item_id: Uuid,
    pub item_name: Option<String>,
    pub item_number: Option<String>,
    pub exception_type: String,
    pub severity: String,
    pub message: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub affected_quantity: Option<String>,
    pub affected_date: Option<chrono::NaiveDate>,
    pub resolution_status: String,
    pub resolution_notes: Option<String>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ===========================================================================
// Workplace Health & Safety (EHS)
// Oracle Fusion Cloud: Environment, Health, and Safety
// ===========================================================================

/// Safety incident (injury, near-miss, property damage, environmental release)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyIncident {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub incident_number: String,
    pub title: String,
    pub description: Option<String>,
    pub incident_type: String,
    pub severity: String,
    pub status: String,
    pub priority: String,
    pub incident_date: chrono::NaiveDate,
    pub incident_time: Option<String>,
    pub location: Option<String>,
    pub facility_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub reported_by_id: Option<Uuid>,
    pub reported_by_name: Option<String>,
    pub assigned_to_id: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub root_cause: Option<String>,
    pub immediate_action: Option<String>,
    pub osha_recordable: bool,
    pub osha_classification: Option<String>,
    pub days_away_from_work: i32,
    pub days_restricted: i32,
    pub body_part: Option<String>,
    pub injury_source: Option<String>,
    pub event_type: Option<String>,
    pub environment_factor: Option<String>,
    pub involved_parties: serde_json::Value,
    pub witness_statements: serde_json::Value,
    pub attachments: serde_json::Value,
    pub resolution_date: Option<chrono::NaiveDate>,
    pub closed_date: Option<chrono::NaiveDate>,
    pub closed_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Hazard identification record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hazard {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub hazard_code: String,
    pub title: String,
    pub description: Option<String>,
    pub hazard_category: String,
    pub risk_level: String,
    pub likelihood: String,
    pub consequence: String,
    pub risk_score: i32,
    pub status: String,
    pub location: Option<String>,
    pub facility_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub identified_by_id: Option<Uuid>,
    pub identified_by_name: Option<String>,
    pub identified_date: chrono::NaiveDate,
    pub mitigation_measures: serde_json::Value,
    pub residual_risk_level: Option<String>,
    pub residual_risk_score: Option<i32>,
    pub review_date: Option<chrono::NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Safety inspection / audit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyInspection {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inspection_number: String,
    pub title: String,
    pub description: Option<String>,
    pub inspection_type: String,
    pub status: String,
    pub priority: String,
    pub scheduled_date: chrono::NaiveDate,
    pub completed_date: Option<chrono::NaiveDate>,
    pub location: Option<String>,
    pub facility_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub inspector_id: Option<Uuid>,
    pub inspector_name: Option<String>,
    pub findings_summary: Option<String>,
    pub total_findings: i32,
    pub critical_findings: i32,
    pub non_conformities: i32,
    pub observations: i32,
    pub score: Option<f64>,
    pub max_score: Option<f64>,
    pub score_pct: Option<f64>,
    pub findings: serde_json::Value,
    pub attachments: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Safety-specific Corrective and Preventive Action (CAPA)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyCorrectiveAction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub action_number: String,
    pub title: String,
    pub description: Option<String>,
    pub action_type: String,
    pub status: String,
    pub priority: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub root_cause: Option<String>,
    pub corrective_action_plan: Option<String>,
    pub preventive_action_plan: Option<String>,
    pub assigned_to_id: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub completed_date: Option<chrono::NaiveDate>,
    pub verified_by: Option<Uuid>,
    pub verified_date: Option<chrono::NaiveDate>,
    pub effectiveness: Option<String>,
    pub facility_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub estimated_cost: Option<f64>,
    pub actual_cost: Option<f64>,
    pub currency_code: Option<String>,
    pub notes: Option<String>,
    pub attachments: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Workplace Health & Safety Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthSafetyDashboard {
    pub organization_id: Uuid,
    pub total_incidents: i64,
    pub open_incidents: i64,
    pub closed_incidents: i64,
    pub critical_incidents: i64,
    pub total_hazards: i64,
    pub open_hazards: i64,
    pub high_risk_hazards: i64,
    pub total_inspections: i64,
    pub open_inspections: i64,
    pub completed_inspections: i64,
    pub total_capa: i64,
    pub open_capa: i64,
    pub overdue_capa: i64,
    pub osha_recordable_count: i64,
    pub days_since_last_incident: i64,
    pub incidents_by_type: serde_json::Value,
    pub incidents_by_severity: serde_json::Value,
    pub hazards_by_risk: serde_json::Value,
    pub inspection_pass_rate: f64,
}

/// Supply Chain Planning Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningDashboard {
    pub organization_id: Uuid,
    pub total_scenarios: i64,
    pub active_scenarios: i64,
    pub total_planned_orders: i64,
    pub unfirm_orders: i64,
    pub firmed_orders: i64,
    pub total_exceptions: i64,
    pub critical_exceptions: i64,
    pub open_exceptions: i64,
    pub total_items_planned: i64,
    pub items_with_shortage: i64,
}

// ════════════════════════════════════════════════════════════════════════════════
// Funds Reservation & Budgetary Control
// (Oracle Fusion: Financials > Budgetary Control > Funds Reservation)
// ════════════════════════════════════════════════════════════════════════════════
//
// Enables organizations to:
// - Reserve funds against approved budgets before actual spending
// - Check fund availability (advisory or absolute control)
// - Consume reserved funds when actual transactions post
// - Release unneeded reservations back to available budget
// - Track fund reservations, consumption, and available balances
//
// This is distinct from the general budget module (which defines budgets)
// and the encumbrance module (general commitments). Budgetary Control
// specifically handles fund checking and reservation workflows.

/// Fund Reservation header
/// Represents a reservation of funds against a budget for an anticipated expenditure.
/// Oracle Fusion equivalent: Budgetary Control > Funds Reservation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundReservation {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated reservation number (e.g., "FR-2024-00001")
    pub reservation_number: String,
    /// Budget definition this reservation is against
    pub budget_id: Uuid,
    /// Budget code (denormalized)
    pub budget_code: String,
    /// Budget version ID (specific version of the budget)
    pub budget_version_id: Option<Uuid>,
    /// Description of why funds are being reserved
    pub description: Option<String>,
    /// Reference to the originating document (e.g., purchase requisition, PO)
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    /// Total reserved amount in budget currency
    pub reserved_amount: f64,
    /// Amount consumed (matched to actual expenditure)
    pub consumed_amount: f64,
    /// Amount released back to available budget
    pub released_amount: f64,
    /// Remaining reserved amount (reserved - consumed - released)
    pub remaining_amount: f64,
    /// Currency code
    pub currency_code: String,
    /// Reservation date
    pub reservation_date: chrono::NaiveDate,
    /// Expiry date (if funds not consumed by this date, auto-release)
    pub expiry_date: Option<chrono::NaiveDate>,
    /// Status: "draft", "active", "`partially_consumed`", "`fully_consumed`", "released", "expired", "cancelled"
    pub status: String,
    /// Control level applied: "advisory", "absolute"
    pub control_level: String,
    /// Fiscal year
    pub fiscal_year: Option<i32>,
    /// Period name (e.g., "Jan-24", "Q1-2024")
    pub period_name: Option<String>,
    /// Requesting department
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    /// Fund check results
    pub fund_check_passed: bool,
    pub fund_check_message: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fund Reservation Line
/// Individual line within a fund reservation, tied to a specific budget line / account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundReservationLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub reservation_id: Uuid,
    /// Line number within the reservation
    pub line_number: i32,
    /// Account code being reserved against
    pub account_code: String,
    /// Account description
    pub account_description: Option<String>,
    /// Budget line ID (specific line in the budget)
    pub budget_line_id: Option<Uuid>,
    /// Department ID
    pub department_id: Option<Uuid>,
    /// Project ID
    pub project_id: Option<Uuid>,
    /// Cost center
    pub cost_center: Option<String>,
    /// Reserved amount on this line
    pub reserved_amount: f64,
    /// Consumed amount
    pub consumed_amount: f64,
    /// Released amount
    pub released_amount: f64,
    /// Remaining amount
    pub remaining_amount: f64,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fund Availability Check result
/// Returned when checking whether funds are available for a given amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundAvailability {
    pub organization_id: Uuid,
    pub budget_id: Uuid,
    pub budget_code: String,
    pub account_code: String,
    /// Original budget amount
    pub budget_amount: f64,
    /// Total reserved (all active reservations)
    pub total_reserved: f64,
    /// Total consumed
    pub total_consumed: f64,
    /// Total released
    pub total_released: f64,
    /// Available balance = `budget_amount` - `total_reserved` + `total_released` - `total_consumed`
    pub available_balance: f64,
    /// Whether the requested amount can be reserved
    pub check_passed: bool,
    /// Control level: "advisory" (warning) or "absolute" (block)
    pub control_level: String,
    /// Human-readable message
    pub message: String,
    /// As-of date for the check
    pub as_of_date: chrono::NaiveDate,
    /// Fiscal year
    pub fiscal_year: Option<i32>,
    /// Period name
    pub period_name: Option<String>,
}

/// Budgetary Control Dashboard
/// Summary of fund reservation activity across the organization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetaryControlDashboard {
    pub organization_id: Uuid,
    pub total_reservations: i64,
    pub active_reservations: i64,
    pub total_reserved_amount: f64,
    pub total_consumed_amount: f64,
    pub total_released_amount: f64,
    pub total_available_amount: f64,
    pub reservations_by_status: serde_json::Value,
    pub top_departments_by_reservation: serde_json::Value,
    pub budget_utilization_pct: f64,
}

