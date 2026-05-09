//! Invoice Matching Module
//!
//! Oracle Fusion Cloud ERP-inspired Invoice Matching for Accounts Payable.
//! Supports 2-way, 3-way, and 4-way matching of supplier invoices against
//! purchase orders and receipts to validate amounts before payment.
//!
//! Features:
//! - 2-way matching: Invoice ↔ Purchase Order
//! - 3-way matching: Invoice ↔ Purchase Order ↔ Receipt
//! - 4-way matching: Invoice ↔ Purchase Order ↔ Receipt ↔ Inspection
//! - Tolerance-based acceptance/rejection
//! - Match exception handling and resolution
//! - Hold/unhold matching with audit trail
//! - Matching dashboard and reporting
//!
//! Oracle Fusion equivalent: Financials > Payables > Invoice Matching

mod engine;

pub use engine::InvoiceMatchingEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Invoice matching header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceMatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub match_number: String,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub purchase_order_id: Uuid,
    pub po_number: Option<String>,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    /// 'two_way', 'three_way', 'four_way'
    pub match_type: String,
    /// 'pending', 'matched', 'partial_match', 'exception', 'overridden', 'cancelled'
    pub status: String,
    pub invoice_amount: String,
    pub po_amount: String,
    pub receipt_amount: Option<String>,
    pub inspection_amount: Option<String>,
    /// Price tolerance percentage (e.g., "2.00" means 2%)
    pub price_tolerance_pct: String,
    /// Quantity tolerance percentage
    pub quantity_tolerance_pct: String,
    /// Amount tolerance (absolute, e.g., "100.00")
    pub amount_tolerance: String,
    pub price_variance: Option<String>,
    pub quantity_variance: Option<String>,
    pub amount_variance: Option<String>,
    pub variance_reason: Option<String>,
    pub receipt_id: Option<Uuid>,
    pub receipt_number: Option<String>,
    pub inspection_id: Option<Uuid>,
    pub inspection_status: Option<String>,
    pub hold_reason: Option<String>,
    pub hold_at: Option<chrono::DateTime<chrono::Utc>>,
    pub hold_by: Option<Uuid>,
    pub override_reason: Option<String>,
    pub overridden_at: Option<chrono::DateTime<chrono::Utc>>,
    pub overridden_by: Option<Uuid>,
    pub matched_at: Option<chrono::DateTime<chrono::Utc>>,
    pub matched_by: Option<Uuid>,
    pub cancelled_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Individual match line (invoice line ↔ PO line)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceMatchLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub match_id: Uuid,
    pub invoice_line_id: Option<Uuid>,
    pub po_line_id: Option<Uuid>,
    pub receipt_line_id: Option<Uuid>,
    pub inspection_line_id: Option<Uuid>,
    pub line_number: i32,
    pub item_description: Option<String>,
    pub invoice_quantity: String,
    pub po_quantity: String,
    pub receipt_quantity: Option<String>,
    pub inspected_quantity: Option<String>,
    pub invoice_unit_price: String,
    pub po_unit_price: String,
    pub invoice_line_amount: String,
    pub po_line_amount: String,
    /// 'matched', 'unmatched', 'exception', 'overridden'
    pub status: String,
    pub price_variance: Option<String>,
    pub quantity_variance: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Matching dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceMatchingDashboard {
    pub total_matches: i32,
    pub matched_count: i32,
    pub pending_count: i32,
    pub exception_count: i32,
    pub overridden_count: i32,
    pub total_invoice_amount: String,
    pub total_variance_amount: String,
    pub matches_by_type: serde_json::Value,
    pub recent_exceptions: serde_json::Value,
}

/// Repository trait
#[async_trait]
pub trait InvoiceMatchingRepository: Send + Sync {
    async fn create_match(&self, org_id: Uuid, match_number: &str, invoice_id: Uuid, invoice_number: Option<&str>, purchase_order_id: Uuid, po_number: Option<&str>, supplier_id: Uuid, supplier_name: &str, match_type: &str, invoice_amount: &str, po_amount: &str, receipt_amount: Option<&str>, inspection_amount: Option<&str>, price_tolerance_pct: &str, quantity_tolerance_pct: &str, amount_tolerance: &str, receipt_id: Option<Uuid>, receipt_number: Option<&str>, inspection_id: Option<Uuid>, inspection_status: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<InvoiceMatch>;
    async fn get_match(&self, id: Uuid) -> AtlasResult<Option<InvoiceMatch>>;
    async fn get_match_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<InvoiceMatch>>;
    async fn list_matches(&self, org_id: Uuid, status: Option<&str>, match_type: Option<&str>, supplier_id: Option<Uuid>) -> AtlasResult<Vec<InvoiceMatch>>;
    async fn update_match_status(&self, id: Uuid, status: &str) -> AtlasResult<InvoiceMatch>;
    async fn update_match_variances(&self, id: Uuid, price_var: Option<&str>, qty_var: Option<&str>, amt_var: Option<&str>, status: &str) -> AtlasResult<()>;
    async fn update_match_hold(&self, id: Uuid, reason: Option<&str>, held_by: Option<Uuid>) -> AtlasResult<InvoiceMatch>;
    async fn update_match_override(&self, id: Uuid, reason: Option<&str>, overridden_by: Option<Uuid>) -> AtlasResult<InvoiceMatch>;
    async fn update_match_matched(&self, id: Uuid, matched_by: Option<Uuid>) -> AtlasResult<InvoiceMatch>;
    async fn cancel_match(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<InvoiceMatch>;
    async fn create_match_line(&self, org_id: Uuid, match_id: Uuid, invoice_line_id: Option<Uuid>, po_line_id: Option<Uuid>, receipt_line_id: Option<Uuid>, inspection_line_id: Option<Uuid>, line_number: i32, item_description: Option<&str>, invoice_qty: &str, po_qty: &str, receipt_qty: Option<&str>, inspected_qty: Option<&str>, invoice_price: &str, po_price: &str, invoice_amt: &str, po_amt: &str, status: &str, price_var: Option<&str>, qty_var: Option<&str>, notes: Option<&str>) -> AtlasResult<InvoiceMatchLine>;
    async fn get_match_line(&self, id: Uuid) -> AtlasResult<Option<InvoiceMatchLine>>;
    async fn list_match_lines(&self, match_id: Uuid) -> AtlasResult<Vec<InvoiceMatchLine>>;
    async fn update_match_line_status(&self, id: Uuid, status: &str) -> AtlasResult<InvoiceMatchLine>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<InvoiceMatchingDashboard>;
}

/// PostgreSQL implementation (stub)
#[allow(dead_code)]
pub struct PostgresInvoiceMatchingRepository { #[allow(dead_code)]
    pool: PgPool }
impl PostgresInvoiceMatchingRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl InvoiceMatchingRepository for PostgresInvoiceMatchingRepository {
    async fn create_match(&self, _: Uuid, _: &str, _: Uuid, _: Option<&str>, _: Uuid, _: Option<&str>, _: Uuid, _: &str, _: &str, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: &str, _: &str, _: &str, _: Option<Uuid>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<InvoiceMatch> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_match(&self, _: Uuid) -> AtlasResult<Option<InvoiceMatch>> { Ok(None) }
    async fn get_match_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<InvoiceMatch>> { Ok(None) }
    async fn list_matches(&self, _: Uuid, _: Option<&str>, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<Vec<InvoiceMatch>> { Ok(vec![]) }
    async fn update_match_status(&self, _: Uuid, _: &str) -> AtlasResult<InvoiceMatch> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_match_variances(&self, _: Uuid, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn update_match_hold(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<InvoiceMatch> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_match_override(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<InvoiceMatch> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_match_matched(&self, _: Uuid, _: Option<Uuid>) -> AtlasResult<InvoiceMatch> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn cancel_match(&self, _: Uuid, _: Option<&str>) -> AtlasResult<InvoiceMatch> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn create_match_line(&self, _: Uuid, _: Uuid, _: Option<Uuid>, _: Option<Uuid>, _: Option<Uuid>, _: Option<Uuid>, _: i32, _: Option<&str>, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: &str, _: &str, _: &str, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>) -> AtlasResult<InvoiceMatchLine> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_match_line(&self, _: Uuid) -> AtlasResult<Option<InvoiceMatchLine>> { Ok(None) }
    async fn list_match_lines(&self, _: Uuid) -> AtlasResult<Vec<InvoiceMatchLine>> { Ok(vec![]) }
    async fn update_match_line_status(&self, _: Uuid, _: &str) -> AtlasResult<InvoiceMatchLine> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<InvoiceMatchingDashboard> { Ok(InvoiceMatchingDashboard { total_matches: 0, matched_count: 0, pending_count: 0, exception_count: 0, overridden_count: 0, total_invoice_amount: "0".into(), total_variance_amount: "0".into(), matches_by_type: serde_json::json!([]), recent_exceptions: serde_json::json!([]) }) }
}
