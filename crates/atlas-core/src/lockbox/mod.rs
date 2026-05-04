//! Lockbox Processing Module
//!
//! Oracle Fusion: AR > Lockbox
//! Automated receipt application from bank lockbox files.

mod engine;
pub use engine::LockboxEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockboxBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub lockbox_number: String,
    pub bank_name: Option<String>,
    pub deposit_date: chrono::NaiveDate,
    pub status: String,
    pub total_amount: String,
    pub total_receipts: i32,
    pub applied_amount: String,
    pub unapplied_amount: String,
    pub on_account_amount: String,
    pub currency_code: String,
    pub source_file_name: Option<String>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockboxReceipt {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub receipt_number: String,
    pub customer_number: Option<String>,
    pub customer_id: Option<Uuid>,
    pub receipt_date: chrono::NaiveDate,
    pub receipt_amount: String,
    pub applied_amount: String,
    pub unapplied_amount: String,
    pub on_account_amount: String,
    pub status: String,
    pub match_type: Option<String>,
    pub remittance_reference: Option<String>,
    pub error_message: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockboxApplication {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_id: Uuid,
    pub invoice_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub applied_amount: String,
    pub application_date: chrono::NaiveDate,
    pub status: String,
    pub applied_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockboxTransmissionFormat {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub format_code: String,
    pub name: String,
    pub description: Option<String>,
    pub format_type: String,
    pub field_delimiter: Option<String>,
    pub record_delimiter: Option<String>,
    pub header_identifier: Option<String>,
    pub detail_identifier: Option<String>,
    pub trailer_identifier: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockboxDashboard {
    pub total_batches: i32,
    pub pending_batches: i32,
    pub completed_batches: i32,
    pub total_receipts: i32,
    pub total_applied_amount: String,
    pub total_unapplied_amount: String,
    pub error_batches: i32,
}

#[async_trait]
pub trait LockboxRepository: Send + Sync {
    async fn create_batch(&self, org_id: Uuid, batch_number: &str, lockbox_number: &str, bank_name: Option<&str>, deposit_date: chrono::NaiveDate, currency_code: &str, source_file_name: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<LockboxBatch>;
    async fn get_batch(&self, org_id: Uuid, batch_number: &str) -> AtlasResult<Option<LockboxBatch>>;
    async fn get_batch_by_id(&self, id: Uuid) -> AtlasResult<Option<LockboxBatch>>;
    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<LockboxBatch>>;
    async fn update_batch_status(&self, id: Uuid, status: &str, error_message: Option<&str>) -> AtlasResult<LockboxBatch>;
    async fn update_batch_amounts(&self, id: Uuid, total_amount: &str, total_receipts: i32, applied: &str, unapplied: &str, on_account: &str) -> AtlasResult<()>;

    async fn create_receipt(&self, org_id: Uuid, batch_id: Uuid, receipt_number: &str, customer_number: Option<&str>, customer_id: Option<Uuid>, receipt_date: chrono::NaiveDate, receipt_amount: &str, remittance_reference: Option<&str>) -> AtlasResult<LockboxReceipt>;
    async fn get_receipt(&self, id: Uuid) -> AtlasResult<Option<LockboxReceipt>>;
    async fn list_receipts_by_batch(&self, batch_id: Uuid) -> AtlasResult<Vec<LockboxReceipt>>;
    async fn update_receipt_status(&self, id: Uuid, status: &str, applied: &str, unapplied: &str, on_account: &str, match_type: Option<&str>, error_message: Option<&str>) -> AtlasResult<LockboxReceipt>;

    async fn create_application(&self, org_id: Uuid, receipt_id: Uuid, invoice_id: Option<Uuid>, invoice_number: Option<&str>, applied_amount: &str, application_date: chrono::NaiveDate, applied_by: Option<Uuid>) -> AtlasResult<LockboxApplication>;
    async fn list_applications_by_receipt(&self, receipt_id: Uuid) -> AtlasResult<Vec<LockboxApplication>>;
    async fn reverse_application(&self, id: Uuid) -> AtlasResult<LockboxApplication>;

    async fn create_format(&self, org_id: Uuid, format_code: &str, name: &str, description: Option<&str>, format_type: &str, field_delimiter: Option<&str>, record_delimiter: Option<&str>, header_id: Option<&str>, detail_id: Option<&str>, trailer_id: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<LockboxTransmissionFormat>;
    async fn list_formats(&self, org_id: Uuid) -> AtlasResult<Vec<LockboxTransmissionFormat>>;
    async fn delete_format(&self, id: Uuid) -> AtlasResult<()>;

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<LockboxDashboard>;
}

#[allow(dead_code)]
pub struct PostgresLockboxRepository { #[allow(dead_code)]
    pool: PgPool }
impl PostgresLockboxRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl LockboxRepository for PostgresLockboxRepository {
    async fn create_batch(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: chrono::NaiveDate, _: &str, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<LockboxBatch> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_batch(&self, _: Uuid, _: &str) -> AtlasResult<Option<LockboxBatch>> { Ok(None) }
    async fn get_batch_by_id(&self, _: Uuid) -> AtlasResult<Option<LockboxBatch>> { Ok(None) }
    async fn list_batches(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<LockboxBatch>> { Ok(vec![]) }
    async fn update_batch_status(&self, _: Uuid, _: &str, _: Option<&str>) -> AtlasResult<LockboxBatch> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_batch_amounts(&self, _: Uuid, _: &str, _: i32, _: &str, _: &str, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn create_receipt(&self, _: Uuid, _: Uuid, _: &str, _: Option<&str>, _: Option<Uuid>, _: chrono::NaiveDate, _: &str, _: Option<&str>) -> AtlasResult<LockboxReceipt> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_receipt(&self, _: Uuid) -> AtlasResult<Option<LockboxReceipt>> { Ok(None) }
    async fn list_receipts_by_batch(&self, _: Uuid) -> AtlasResult<Vec<LockboxReceipt>> { Ok(vec![]) }
    async fn update_receipt_status(&self, _: Uuid, _: &str, _: &str, _: &str, _: &str, _: Option<&str>, _: Option<&str>) -> AtlasResult<LockboxReceipt> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn create_application(&self, _: Uuid, _: Uuid, _: Option<Uuid>, _: Option<&str>, _: &str, _: chrono::NaiveDate, _: Option<Uuid>) -> AtlasResult<LockboxApplication> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_applications_by_receipt(&self, _: Uuid) -> AtlasResult<Vec<LockboxApplication>> { Ok(vec![]) }
    async fn reverse_application(&self, _: Uuid) -> AtlasResult<LockboxApplication> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn create_format(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<LockboxTransmissionFormat> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_formats(&self, _: Uuid) -> AtlasResult<Vec<LockboxTransmissionFormat>> { Ok(vec![]) }
    async fn delete_format(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<LockboxDashboard> { Ok(LockboxDashboard { total_batches: 0, pending_batches: 0, completed_batches: 0, total_receipts: 0, total_applied_amount: "0".into(), total_unapplied_amount: "0".into(), error_batches: 0 }) }
}
