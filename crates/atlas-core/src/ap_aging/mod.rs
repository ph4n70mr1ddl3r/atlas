//! AP Aging Analysis Module
//!
//! Oracle Fusion Cloud ERP-inspired Accounts Payable Aging analysis.
//! Manages AP aging definitions, buckets, snapshots, and summary reporting.
//!
//! Oracle Fusion equivalent: Financials > Payables > Aging Reports

mod engine;
pub use engine::ApAgingEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

// Local type definitions

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApAgingDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub definition_code: String,
    pub name: String,
    pub description: Option<String>,
    pub aging_basis: String,
    pub num_buckets: i32,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApAgingBucket {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub definition_id: Uuid,
    pub bucket_number: i32,
    pub name: String,
    pub from_days: i32,
    pub to_days: Option<i32>,
    pub display_order: i32,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApAgingSnapshot {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub definition_id: Uuid,
    pub snapshot_date: chrono::NaiveDate,
    pub as_of_date: chrono::NaiveDate,
    pub total_open_amount: String,
    pub total_overdue_amount: String,
    pub total_past_due_count: i32,
    pub currency_code: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApAgingSnapshotLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub snapshot_id: Uuid,
    pub supplier_id: Option<Uuid>,
    pub supplier_number: String,
    pub supplier_name: Option<String>,
    pub invoice_id: Option<Uuid>,
    pub invoice_number: String,
    pub invoice_date: chrono::NaiveDate,
    pub due_date: chrono::NaiveDate,
    pub original_amount: String,
    pub open_amount: String,
    pub days_past_due: i32,
    pub bucket_number: i32,
    pub bucket_name: String,
    pub currency_code: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApAgingSummary {
    pub bucket_number: i32,
    pub bucket_name: String,
    pub supplier_count: i32,
    pub invoice_count: i32,
    pub total_open_amount: String,
    pub total_overdue_amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApAgingDashboard {
    pub total_definitions: i32,
    pub total_snapshots: i32,
    pub total_open_payables: String,
    pub total_overdue: String,
    pub overdue_count: i32,
    pub avg_days_past_due: String,
}

/// Repository trait
#[async_trait]
pub trait ApAgingRepository: Send + Sync {
    async fn create_definition(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, aging_basis: &str, num_buckets: i32, created_by: Option<Uuid>) -> AtlasResult<ApAgingDefinition>;
    async fn get_definition(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ApAgingDefinition>>;
    async fn get_definition_by_id(&self, id: Uuid) -> AtlasResult<Option<ApAgingDefinition>>;
    async fn list_definitions(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ApAgingDefinition>>;
    async fn update_definition_status(&self, id: Uuid, status: &str) -> AtlasResult<ApAgingDefinition>;
    async fn delete_definition(&self, id: Uuid) -> AtlasResult<()>;
    async fn create_bucket(&self, org_id: Uuid, definition_id: Uuid, bucket_number: i32, name: &str, from_days: i32, to_days: Option<i32>, display_order: i32) -> AtlasResult<ApAgingBucket>;
    async fn list_buckets(&self, definition_id: Uuid) -> AtlasResult<Vec<ApAgingBucket>>;
    async fn create_snapshot(&self, org_id: Uuid, definition_id: Uuid, as_of_date: chrono::NaiveDate, currency_code: &str, created_by: Option<Uuid>) -> AtlasResult<ApAgingSnapshot>;
    async fn get_snapshot(&self, id: Uuid) -> AtlasResult<Option<ApAgingSnapshot>>;
    async fn list_snapshots(&self, org_id: Uuid, definition_id: Option<Uuid>) -> AtlasResult<Vec<ApAgingSnapshot>>;
    async fn update_snapshot_totals(&self, id: Uuid, total_open: &str, total_overdue: &str, past_due_count: i32) -> AtlasResult<()>;
    async fn create_snapshot_line(&self, org_id: Uuid, snapshot_id: Uuid, supplier_id: Option<Uuid>, supplier_number: &str, supplier_name: Option<&str>, invoice_id: Option<Uuid>, invoice_number: &str, invoice_date: chrono::NaiveDate, due_date: chrono::NaiveDate, original_amount: &str, open_amount: &str, days_past_due: i32, bucket_number: i32, bucket_name: &str, currency_code: &str) -> AtlasResult<ApAgingSnapshotLine>;
    async fn list_snapshot_lines(&self, snapshot_id: Uuid) -> AtlasResult<Vec<ApAgingSnapshotLine>>;
    async fn get_aging_summary(&self, snapshot_id: Uuid) -> AtlasResult<Vec<ApAgingSummary>>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ApAgingDashboard>;
}

/// PostgreSQL implementation
pub struct PostgresApAgingRepository { pool: PgPool }
impl PostgresApAgingRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl ApAgingRepository for PostgresApAgingRepository {
    async fn create_definition(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: i32, _: Option<Uuid>) -> AtlasResult<ApAgingDefinition> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_definition(&self, _: Uuid, _: &str) -> AtlasResult<Option<ApAgingDefinition>> { Ok(None) }
    async fn get_definition_by_id(&self, _: Uuid) -> AtlasResult<Option<ApAgingDefinition>> { Ok(None) }
    async fn list_definitions(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<ApAgingDefinition>> { Ok(vec![]) }
    async fn update_definition_status(&self, _: Uuid, _: &str) -> AtlasResult<ApAgingDefinition> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn delete_definition(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn create_bucket(&self, _: Uuid, _: Uuid, _: i32, _: &str, _: i32, _: Option<i32>, _: i32) -> AtlasResult<ApAgingBucket> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_buckets(&self, _: Uuid) -> AtlasResult<Vec<ApAgingBucket>> { Ok(vec![]) }
    async fn create_snapshot(&self, _: Uuid, _: Uuid, _: chrono::NaiveDate, _: &str, _: Option<Uuid>) -> AtlasResult<ApAgingSnapshot> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_snapshot(&self, _: Uuid) -> AtlasResult<Option<ApAgingSnapshot>> { Ok(None) }
    async fn list_snapshots(&self, _: Uuid, _: Option<Uuid>) -> AtlasResult<Vec<ApAgingSnapshot>> { Ok(vec![]) }
    async fn update_snapshot_totals(&self, _: Uuid, _: &str, _: &str, _: i32) -> AtlasResult<()> { Ok(()) }
    async fn create_snapshot_line(&self, _: Uuid, _: Uuid, _: Option<Uuid>, _: &str, _: Option<&str>, _: Option<Uuid>, _: &str, _: chrono::NaiveDate, _: chrono::NaiveDate, _: &str, _: &str, _: i32, _: i32, _: &str, _: &str) -> AtlasResult<ApAgingSnapshotLine> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_snapshot_lines(&self, _: Uuid) -> AtlasResult<Vec<ApAgingSnapshotLine>> { Ok(vec![]) }
    async fn get_aging_summary(&self, _: Uuid) -> AtlasResult<Vec<ApAgingSummary>> { Ok(vec![]) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<ApAgingDashboard> { Ok(ApAgingDashboard { total_definitions: 0, total_snapshots: 0, total_open_payables: "0".into(), total_overdue: "0".into(), overdue_count: 0, avg_days_past_due: "0".into() }) }
}
