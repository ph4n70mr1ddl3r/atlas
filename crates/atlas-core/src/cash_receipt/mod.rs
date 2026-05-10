//! Cash Receipt Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Cash Receipt Management for Accounts Receivable.
//! Manages the full lifecycle of customer cash receipts:
//! - Create receipt batches to group related receipts
//! - Record individual cash receipts with customer and payment details
//! - Apply receipts to open invoices
//! - Unapply receipt applications
//! - Reverse receipts
//! - Track receipt statuses through the full workflow
//!
//! Receipt statuses: unidentified → identified → unapplied → applied/partially_applied → reversed
//! Batch statuses: draft → confirmed → closed → cancelled
//!
//! Oracle Fusion equivalent: Financials > Receivables > Receipts

mod engine;

pub use engine::CashReceiptEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Receipt batch definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub batch_name: String,
    pub description: Option<String>,
    pub receipt_method: String,
    pub bank_account_id: Option<Uuid>,
    pub status: String,
    pub currency_code: String,
    pub total_amount: String,
    pub receipt_count: i32,
    pub gl_posting_date: Option<chrono::NaiveDate>,
    pub posted_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Individual cash receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashReceipt {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Option<Uuid>,
    pub receipt_number: String,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub customer_account_number: Option<String>,
    pub payment_method: String,
    pub status: String,
    pub amount: String,
    pub applied_amount: String,
    pub unapplied_amount: String,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub receipt_date: chrono::NaiveDate,
    pub maturity_date: Option<chrono::NaiveDate>,
    pub reference_number: Option<String>,
    pub bank_name: Option<String>,
    pub bank_branch: Option<String>,
    pub deposit_date: Option<chrono::NaiveDate>,
    pub clearance_status: Option<String>,
    pub notes: Option<String>,
    pub reversal_reason: Option<String>,
    pub reversed_from: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Receipt application (linking a receipt to an invoice)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptApplication {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_id: Uuid,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub applied_amount: String,
    pub discount_taken: String,
    pub status: String,
    pub application_date: chrono::NaiveDate,
    pub gl_date: Option<chrono::NaiveDate>,
    pub applied_by: Option<Uuid>,
    pub reversed_by: Option<Uuid>,
    pub reversal_date: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Cash receipt dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashReceiptDashboard {
    pub total_batches: i64,
    pub draft_batches: i64,
    pub confirmed_batches: i64,
    pub total_receipts: i64,
    pub applied_receipts: i64,
    pub unapplied_receipts: i64,
    pub partially_applied_receipts: i64,
    pub reversed_receipts: i64,
    pub total_receipt_amount: String,
    pub total_applied_amount: String,
    pub total_unapplied_amount: String,
}

/// Repository trait for cash receipt persistence
#[async_trait]
pub trait CashReceiptRepository: Send + Sync {
    // Batches
    async fn create_batch(
        &self, org_id: Uuid, batch_number: &str, batch_name: &str,
        description: Option<&str>, receipt_method: &str,
        bank_account_id: Option<Uuid>, currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptBatch>;

    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<ReceiptBatch>>;
    async fn get_batch_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<ReceiptBatch>>;
    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ReceiptBatch>>;
    async fn update_batch_status(&self, id: Uuid, status: &str) -> AtlasResult<ReceiptBatch>;
    async fn update_batch_totals(&self, id: Uuid, total_amount: &str, receipt_count: i32) -> AtlasResult<()>;
    async fn delete_batch(&self, id: Uuid) -> AtlasResult<()>;

    // Receipts
    async fn create_receipt(
        &self, org_id: Uuid, batch_id: Option<Uuid>,
        receipt_number: &str, customer_id: Uuid,
        customer_name: Option<&str>, customer_account_number: Option<&str>,
        payment_method: &str, amount: &str, currency_code: &str,
        exchange_rate: Option<&str>, receipt_date: chrono::NaiveDate,
        maturity_date: Option<chrono::NaiveDate>,
        reference_number: Option<&str>,
        bank_name: Option<&str>, bank_branch: Option<&str>,
        deposit_date: Option<chrono::NaiveDate>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<CashReceipt>;

    async fn get_receipt(&self, id: Uuid) -> AtlasResult<Option<CashReceipt>>;
    async fn get_receipt_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<CashReceipt>>;
    async fn list_receipts(&self, org_id: Uuid, batch_id: Option<Uuid>, customer_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<CashReceipt>>;
    async fn update_receipt_status(&self, id: Uuid, status: &str) -> AtlasResult<CashReceipt>;
    async fn update_receipt_amounts(&self, id: Uuid, applied_amount: &str, unapplied_amount: &str) -> AtlasResult<CashReceipt>;
    async fn update_receipt_reversal(&self, id: Uuid, reason: &str, reversed_from: Option<Uuid>) -> AtlasResult<CashReceipt>;

    // Applications
    async fn create_application(
        &self, org_id: Uuid, receipt_id: Uuid, invoice_id: Uuid,
        invoice_number: Option<&str>, applied_amount: &str, discount_taken: &str,
        application_date: chrono::NaiveDate, gl_date: Option<chrono::NaiveDate>,
        applied_by: Option<Uuid>,
    ) -> AtlasResult<ReceiptApplication>;

    async fn get_application(&self, id: Uuid) -> AtlasResult<Option<ReceiptApplication>>;
    async fn list_applications(&self, receipt_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ReceiptApplication>>;
    async fn update_application_status(&self, id: Uuid, status: &str, reversed_by: Option<Uuid>) -> AtlasResult<ReceiptApplication>;
    async fn sum_applied_for_receipt(&self, receipt_id: Uuid) -> AtlasResult<String>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CashReceiptDashboard>;
}

/// PostgreSQL stub implementation
#[allow(dead_code)]
pub struct PostgresCashReceiptRepository { #[allow(dead_code)] pool: PgPool }
impl PostgresCashReceiptRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl CashReceiptRepository for PostgresCashReceiptRepository {
    async fn create_batch(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: Option<Uuid>, _: &str, _: Option<Uuid>) -> AtlasResult<ReceiptBatch> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_batch(&self, _: Uuid) -> AtlasResult<Option<ReceiptBatch>> { Ok(None) }
    async fn get_batch_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<ReceiptBatch>> { Ok(None) }
    async fn list_batches(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<ReceiptBatch>> { Ok(vec![]) }
    async fn update_batch_status(&self, _: Uuid, _: &str) -> AtlasResult<ReceiptBatch> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_batch_totals(&self, _: Uuid, _: &str, _: i32) -> AtlasResult<()> { Ok(()) }
    async fn delete_batch(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }

    async fn create_receipt(&self, _: Uuid, _: Option<Uuid>, _: &str, _: Uuid, _: Option<&str>, _: Option<&str>, _: &str, _: &str, _: &str, _: Option<&str>, _: chrono::NaiveDate, _: Option<chrono::NaiveDate>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<chrono::NaiveDate>, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<CashReceipt> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_receipt(&self, _: Uuid) -> AtlasResult<Option<CashReceipt>> { Ok(None) }
    async fn get_receipt_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<CashReceipt>> { Ok(None) }
    async fn list_receipts(&self, _: Uuid, _: Option<Uuid>, _: Option<Uuid>, _: Option<&str>) -> AtlasResult<Vec<CashReceipt>> { Ok(vec![]) }
    async fn update_receipt_status(&self, _: Uuid, _: &str) -> AtlasResult<CashReceipt> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_receipt_amounts(&self, _: Uuid, _: &str, _: &str) -> AtlasResult<CashReceipt> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_receipt_reversal(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<CashReceipt> { Err(AtlasError::EntityNotFound("Mock".into())) }

    async fn create_application(&self, _: Uuid, _: Uuid, _: Uuid, _: Option<&str>, _: &str, _: &str, _: chrono::NaiveDate, _: Option<chrono::NaiveDate>, _: Option<Uuid>) -> AtlasResult<ReceiptApplication> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_application(&self, _: Uuid) -> AtlasResult<Option<ReceiptApplication>> { Ok(None) }
    async fn list_applications(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<ReceiptApplication>> { Ok(vec![]) }
    async fn update_application_status(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<ReceiptApplication> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn sum_applied_for_receipt(&self, _: Uuid) -> AtlasResult<String> { Ok("0.00".into()) }

    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<CashReceiptDashboard> {
        Ok(CashReceiptDashboard {
            total_batches: 0, draft_batches: 0, confirmed_batches: 0,
            total_receipts: 0, applied_receipts: 0, unapplied_receipts: 0,
            partially_applied_receipts: 0, reversed_receipts: 0,
            total_receipt_amount: "0.00".into(), total_applied_amount: "0.00".into(),
            total_unapplied_amount: "0.00".into(),
        })
    }
}
