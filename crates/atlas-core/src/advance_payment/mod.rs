//! Advance Payment Module (Supplier Prepayments)
//!
//! Oracle Fusion Cloud ERP-inspired Advance Payment management.
//! Manages supplier prepayments, application to invoices, and settlement.
//!
//! Features:
//! - Advance payment creation and approval workflow
//! - Application of advances to supplier invoices
//! - Partial/full application tracking
//! - Unapplied advance management
//! - Advance aging and reporting
//!
//! Oracle Fusion equivalent: Financials > Accounts Payable > Advance Payments

mod engine;

pub use engine::AdvancePaymentEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Advance payment header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancePayment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub advance_number: String,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub supplier_site_id: Option<Uuid>,
    pub description: Option<String>,
    /// 'draft', 'approved', 'paid', 'partially_applied', 'fully_applied', 'cancelled'
    pub status: String,
    pub currency_code: String,
    pub advance_amount: String,
    pub applied_amount: String,
    pub unapplied_amount: String,
    pub exchange_rate: Option<String>,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    /// GL account for the advance (prepayment asset)
    pub prepayment_account_code: Option<String>,
    /// GL account for the supplier liability
    pub liability_account_code: Option<String>,
    pub advance_date: chrono::NaiveDate,
    pub payment_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub expiration_date: Option<chrono::NaiveDate>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub paid_by: Option<Uuid>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancelled_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Application of an advance payment to an invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvanceApplication {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub advance_id: Uuid,
    pub advance_number: Option<String>,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub applied_amount: String,
    pub application_date: chrono::NaiveDate,
    /// 'applied', 'unapplied', 'reversed'
    pub status: String,
    pub gl_account_code: Option<String>,
    pub reversed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reversed_by: Option<Uuid>,
    pub reversal_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub applied_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Advance payment dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancePaymentDashboard {
    pub total_advances: i32,
    pub draft_advances: i32,
    pub open_advances: i32,
    pub total_advance_amount: String,
    pub total_applied_amount: String,
    pub total_unapplied_amount: String,
    pub advances_by_supplier: serde_json::Value,
    pub aging_buckets: serde_json::Value,
}

/// Repository trait
#[async_trait]
pub trait AdvancePaymentRepository: Send + Sync {
    async fn create_advance(&self, org_id: Uuid, advance_number: &str, supplier_id: Uuid, supplier_name: &str, supplier_site_id: Option<Uuid>, description: Option<&str>, currency_code: &str, advance_amount: &str, exchange_rate: Option<&str>, payment_method: Option<&str>, prepayment_account_code: Option<&str>, liability_account_code: Option<&str>, advance_date: chrono::NaiveDate, due_date: Option<chrono::NaiveDate>, expiration_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>) -> AtlasResult<AdvancePayment>;
    async fn get_advance(&self, id: Uuid) -> AtlasResult<Option<AdvancePayment>>;
    async fn get_advance_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<AdvancePayment>>;
    async fn list_advances(&self, org_id: Uuid, status: Option<&str>, supplier_id: Option<Uuid>) -> AtlasResult<Vec<AdvancePayment>>;
    async fn update_advance_status(&self, id: Uuid, status: &str) -> AtlasResult<AdvancePayment>;
    async fn update_advance_amounts(&self, id: Uuid, applied: &str, unapplied: &str, status: &str) -> AtlasResult<()>;
    async fn update_payment_info(&self, id: Uuid, payment_ref: Option<&str>, paid_by: Option<Uuid>) -> AtlasResult<AdvancePayment>;
    async fn create_application(&self, org_id: Uuid, advance_id: Uuid, advance_number: Option<&str>, invoice_id: Uuid, invoice_number: Option<&str>, applied_amount: &str, application_date: chrono::NaiveDate, gl_account_code: Option<&str>, applied_by: Option<Uuid>) -> AtlasResult<AdvanceApplication>;
    async fn get_application(&self, id: Uuid) -> AtlasResult<Option<AdvanceApplication>>;
    async fn list_applications_by_advance(&self, advance_id: Uuid) -> AtlasResult<Vec<AdvanceApplication>>;
    async fn list_applications_by_invoice(&self, invoice_id: Uuid) -> AtlasResult<Vec<AdvanceApplication>>;
    async fn update_application_status(&self, id: Uuid, status: &str, reversed_by: Option<Uuid>, reversal_reason: Option<&str>) -> AtlasResult<AdvanceApplication>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<AdvancePaymentDashboard>;
}

/// PostgreSQL implementation (stub)
#[allow(dead_code)]
pub struct PostgresAdvancePaymentRepository { #[allow(dead_code)]
    pool: PgPool }
impl PostgresAdvancePaymentRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl AdvancePaymentRepository for PostgresAdvancePaymentRepository {
    async fn create_advance(&self, _: Uuid, _: &str, _: Uuid, _: &str, _: Option<Uuid>, _: Option<&str>, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: chrono::NaiveDate, _: Option<chrono::NaiveDate>, _: Option<chrono::NaiveDate>, _: Option<Uuid>) -> AtlasResult<AdvancePayment> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_advance(&self, _: Uuid) -> AtlasResult<Option<AdvancePayment>> { Ok(None) }
    async fn get_advance_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<AdvancePayment>> { Ok(None) }
    async fn list_advances(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<Vec<AdvancePayment>> { Ok(vec![]) }
    async fn update_advance_status(&self, _: Uuid, _: &str) -> AtlasResult<AdvancePayment> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_advance_amounts(&self, _: Uuid, _: &str, _: &str, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn update_payment_info(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<AdvancePayment> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn create_application(&self, _: Uuid, _: Uuid, _: Option<&str>, _: Uuid, _: Option<&str>, _: &str, _: chrono::NaiveDate, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<AdvanceApplication> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_application(&self, _: Uuid) -> AtlasResult<Option<AdvanceApplication>> { Ok(None) }
    async fn list_applications_by_advance(&self, _: Uuid) -> AtlasResult<Vec<AdvanceApplication>> { Ok(vec![]) }
    async fn list_applications_by_invoice(&self, _: Uuid) -> AtlasResult<Vec<AdvanceApplication>> { Ok(vec![]) }
    async fn update_application_status(&self, _: Uuid, _: &str, _: Option<Uuid>, _: Option<&str>) -> AtlasResult<AdvanceApplication> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<AdvancePaymentDashboard> { Ok(AdvancePaymentDashboard { total_advances: 0, draft_advances: 0, open_advances: 0, total_advance_amount: "0".into(), total_applied_amount: "0".into(), total_unapplied_amount: "0".into(), advances_by_supplier: serde_json::json!([]), aging_buckets: serde_json::json!([]) }) }
}
