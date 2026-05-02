//! Receipt Write-Off Module
//!
//! Oracle Fusion Cloud ERP-inspired Receipt Write-Off processing.
//! Manages write-off of unapplied or partially applied receipts
//! with approval workflows and GL posting.
//!
//! Oracle Fusion equivalent: Financials > Receivables > Receipts > Write-Off

mod engine;
pub use engine::ReceiptWriteOffEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteOffReason {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub reason_code: String,
    pub name: String,
    pub description: Option<String>,
    pub default_gl_account: Option<String>,
    pub requires_approval: bool,
    pub max_auto_approve_amount: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteOffRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_number: String,
    pub receipt_id: Uuid,
    pub receipt_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub write_off_amount: String,
    pub currency_code: String,
    pub reason_id: Uuid,
    pub reason_code: String,
    pub comments: Option<String>,
    pub status: String,
    pub gl_account_code: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub journal_entry_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteOffBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub total_amount: String,
    pub total_count: i32,
    pub currency_code: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteOffPolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub min_amount: String,
    pub max_amount: String,
    pub requires_approval: bool,
    pub auto_approve_below: Option<String>,
    pub default_gl_account: Option<String>,
    pub aging_threshold_days: Option<i32>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteOffDashboard {
    pub total_requests: i32,
    pub pending_approval: i32,
    pub approved: i32,
    pub rejected: i32,
    pub posted: i32,
    pub total_write_off_amount: String,
    pub total_by_reason: serde_json::Value,
}

/// Repository trait
#[async_trait]
pub trait ReceiptWriteOffRepository: Send + Sync {
    // Reasons
    async fn create_reason(&self, org_id: Uuid, code: &str, name: &str, desc: Option<&str>, gl: Option<&str>, requires_approval: bool, max_auto: Option<&str>, cb: Option<Uuid>) -> AtlasResult<WriteOffReason>;
    async fn get_reason(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<WriteOffReason>>;
    async fn get_reason_by_id(&self, id: Uuid) -> AtlasResult<Option<WriteOffReason>>;
    async fn list_reasons(&self, org_id: Uuid) -> AtlasResult<Vec<WriteOffReason>>;
    async fn delete_reason(&self, id: Uuid) -> AtlasResult<()>;

    // Requests
    async fn create_request(&self, org_id: Uuid, rn: &str, receipt_id: Uuid, receipt_number: &str, customer_id: Option<Uuid>, customer_number: Option<&str>, amount: &str, currency: &str, reason_id: Uuid, reason_code: &str, comments: Option<&str>, gl: Option<&str>, status: &str, cb: Option<Uuid>) -> AtlasResult<WriteOffRequest>;
    async fn get_request(&self, id: Uuid) -> AtlasResult<Option<WriteOffRequest>>;
    async fn get_request_by_number(&self, org_id: Uuid, rn: &str) -> AtlasResult<Option<WriteOffRequest>>;
    async fn list_requests(&self, org_id: Uuid, status: Option<&str>, reason_id: Option<Uuid>) -> AtlasResult<Vec<WriteOffRequest>>;
    async fn update_request_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>, posted_by: Option<Uuid>, je_id: Option<Uuid>) -> AtlasResult<WriteOffRequest>;

    // Batches
    async fn create_batch(&self, org_id: Uuid, bn: &str, name: &str, desc: Option<&str>, cc: &str, cb: Option<Uuid>) -> AtlasResult<WriteOffBatch>;
    async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<WriteOffBatch>>;
    async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<WriteOffBatch>>;
    async fn update_batch_status(&self, id: Uuid, status: &str, ab: Option<Uuid>) -> AtlasResult<WriteOffBatch>;
    async fn update_batch_totals(&self, id: Uuid, amount: &str, count: i32) -> AtlasResult<()>;

    // Policy
    async fn create_policy(&self, org_id: Uuid, name: &str, desc: Option<&str>, min: &str, max: &str, requires_approval: bool, auto_approve: Option<&str>, gl: Option<&str>, aging_days: Option<i32>, cb: Option<Uuid>) -> AtlasResult<WriteOffPolicy>;
    async fn list_policies(&self, org_id: Uuid) -> AtlasResult<Vec<WriteOffPolicy>>;
    async fn get_active_policy(&self, org_id: Uuid, amount: &str) -> AtlasResult<Option<WriteOffPolicy>>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<WriteOffDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresReceiptWriteOffRepository { pool: PgPool }
impl PostgresReceiptWriteOffRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl ReceiptWriteOffRepository for PostgresReceiptWriteOffRepository {
    async fn create_reason(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: bool, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<WriteOffReason> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_reason(&self, _: Uuid, _: &str) -> AtlasResult<Option<WriteOffReason>> { Ok(None) }
    async fn get_reason_by_id(&self, _: Uuid) -> AtlasResult<Option<WriteOffReason>> { Ok(None) }
    async fn list_reasons(&self, _: Uuid) -> AtlasResult<Vec<WriteOffReason>> { Ok(vec![]) }
    async fn delete_reason(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn create_request(&self, _: Uuid, _: &str, _: Uuid, _: &str, _: Option<Uuid>, _: Option<&str>, _: &str, _: &str, _: Uuid, _: &str, _: Option<&str>, _: Option<&str>, _: &str, _: Option<Uuid>) -> AtlasResult<WriteOffRequest> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_request(&self, _: Uuid) -> AtlasResult<Option<WriteOffRequest>> { Ok(None) }
    async fn get_request_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<WriteOffRequest>> { Ok(None) }
    async fn list_requests(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<Vec<WriteOffRequest>> { Ok(vec![]) }
    async fn update_request_status(&self, _: Uuid, _: &str, _: Option<Uuid>, _: Option<Uuid>, _: Option<Uuid>) -> AtlasResult<WriteOffRequest> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn create_batch(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: Option<Uuid>) -> AtlasResult<WriteOffBatch> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_batch(&self, _: Uuid) -> AtlasResult<Option<WriteOffBatch>> { Ok(None) }
    async fn list_batches(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<WriteOffBatch>> { Ok(vec![]) }
    async fn update_batch_status(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<WriteOffBatch> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_batch_totals(&self, _: Uuid, _: &str, _: i32) -> AtlasResult<()> { Ok(()) }
    async fn create_policy(&self, _: Uuid, _: &str, _: Option<&str>, _: &str, _: &str, _: bool, _: Option<&str>, _: Option<&str>, _: Option<i32>, _: Option<Uuid>) -> AtlasResult<WriteOffPolicy> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_policies(&self, _: Uuid) -> AtlasResult<Vec<WriteOffPolicy>> { Ok(vec![]) }
    async fn get_active_policy(&self, _: Uuid, _: &str) -> AtlasResult<Option<WriteOffPolicy>> { Ok(None) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<WriteOffDashboard> {
        Ok(WriteOffDashboard { total_requests: 0, pending_approval: 0, approved: 0, rejected: 0, posted: 0, total_write_off_amount: "0".into(), total_by_reason: serde_json::json!([]) })
    }
}
