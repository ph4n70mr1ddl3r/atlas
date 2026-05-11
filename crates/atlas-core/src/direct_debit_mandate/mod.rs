//! Direct Debit Mandate Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Direct Debit Mandate Management for Accounts Receivable.
//! Manages customer authorizations for automatic bank collections:
//! - Create direct debit mandates with customer bank details
//! - Activate mandates for collection
//! - Record collections (first, recurring, final) against mandates
//! - Track mandate lifecycle (draft → active → used → expired/cancelled/revoked)
//! - Reverse failed collections
//! - Monitor mandate compliance and status
//!
//! Mandate statuses: draft → active → used → expired/cancelled/revoked
//! Collection statuses: pending → submitted → completed/failed → returned/reversed
//!
//! Oracle Fusion equivalent: Financials > Receivables > Direct Debit Mandates

mod engine;

pub use engine::DirectDebitMandateEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Direct debit mandate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectDebitMandate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub mandate_number: String,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub customer_account_number: Option<String>,
    pub mandate_type: String,
    pub status: String,
    pub bank_account_holder: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_account_number_type: Option<String>,
    pub bank_code: Option<String>,
    pub bank_code_type: Option<String>,
    pub bank_name: Option<String>,
    pub bank_branch: Option<String>,
    pub creditor_scheme_id: Option<String>,
    pub creditor_name: Option<String>,
    pub mandate_reference: Option<String>,
    pub mandate_date: chrono::NaiveDate,
    pub activation_date: Option<chrono::NaiveDate>,
    pub first_collection_date: Option<chrono::NaiveDate>,
    pub last_collection_date: Option<chrono::NaiveDate>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub cancellation_date: Option<chrono::NaiveDate>,
    pub collection_count: i32,
    pub total_collected: String,
    pub next_collection_type: Option<String>,
    pub max_collection_amount: Option<String>,
    pub currency_code: String,
    pub status_reason: Option<String>,
    pub authorization_reference: Option<String>,
    pub authorization_method: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Mandate collection record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MandateCollection {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub mandate_id: Uuid,
    pub collection_number: String,
    pub collection_type: String,
    pub status: String,
    pub amount: String,
    pub currency_code: String,
    pub invoice_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub receipt_id: Option<Uuid>,
    pub scheduled_date: chrono::NaiveDate,
    pub execution_date: Option<chrono::NaiveDate>,
    pub settlement_date: Option<chrono::NaiveDate>,
    pub return_reason_code: Option<String>,
    pub return_reason_text: Option<String>,
    pub bank_reference: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Direct debit dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectDebitDashboard {
    pub total_mandates: i64,
    pub draft_mandates: i64,
    pub active_mandates: i64,
    pub used_mandates: i64,
    pub cancelled_mandates: i64,
    pub expired_mandates: i64,
    pub revoked_mandates: i64,
    pub total_collections: i64,
    pub completed_collections: i64,
    pub failed_collections: i64,
    pub pending_collections: i64,
    pub total_collected_amount: String,
    pub pending_amount: String,
}

/// Repository trait for direct debit mandate persistence
#[async_trait]
pub trait DirectDebitMandateRepository: Send + Sync {
    // Mandates
    async fn create_mandate(
        &self, org_id: Uuid, mandate_number: &str, customer_id: Uuid,
        customer_name: Option<&str>, customer_account_number: Option<&str>,
        mandate_type: &str, bank_account_holder: Option<&str>,
        bank_account_number: Option<&str>, bank_account_number_type: Option<&str>,
        bank_code: Option<&str>, bank_code_type: Option<&str>,
        bank_name: Option<&str>, bank_branch: Option<&str>,
        creditor_scheme_id: Option<&str>, creditor_name: Option<&str>,
        mandate_reference: Option<&str>, mandate_date: chrono::NaiveDate,
        expiry_date: Option<chrono::NaiveDate>,
        max_collection_amount: Option<&str>, currency_code: &str,
        authorization_reference: Option<&str>, authorization_method: Option<&str>,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<DirectDebitMandate>;

    async fn get_mandate(&self, id: Uuid) -> AtlasResult<Option<DirectDebitMandate>>;
    async fn get_mandate_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<DirectDebitMandate>>;
    async fn list_mandates(&self, org_id: Uuid, customer_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<DirectDebitMandate>>;
    async fn update_mandate_status(&self, id: Uuid, status: &str, reason: Option<&str>) -> AtlasResult<DirectDebitMandate>;
    async fn update_mandate_activation(&self, id: Uuid, activation_date: chrono::NaiveDate) -> AtlasResult<DirectDebitMandate>;
    async fn update_mandate_collection_stats(&self, id: Uuid, collection_count: i32, total_collected: &str, last_collection_date: Option<chrono::NaiveDate>, next_type: &str) -> AtlasResult<()>;
    async fn update_mandate_expiry(&self, id: Uuid, expiry_date: chrono::NaiveDate) -> AtlasResult<DirectDebitMandate>;

    // Collections
    async fn create_collection(
        &self, org_id: Uuid, mandate_id: Uuid, collection_number: &str,
        collection_type: &str, amount: &str, currency_code: &str,
        invoice_id: Option<Uuid>, invoice_number: Option<&str>,
        receipt_id: Option<Uuid>, scheduled_date: chrono::NaiveDate,
        notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<MandateCollection>;

    async fn get_collection(&self, id: Uuid) -> AtlasResult<Option<MandateCollection>>;
    async fn list_collections(&self, mandate_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<MandateCollection>>;
    async fn update_collection_status(&self, id: Uuid, status: &str, bank_reference: Option<&str>) -> AtlasResult<MandateCollection>;
    async fn update_collection_return(&self, id: Uuid, reason_code: &str, reason_text: Option<&str>) -> AtlasResult<MandateCollection>;
    async fn sum_collected_for_mandate(&self, mandate_id: Uuid) -> AtlasResult<String>;
    async fn count_collections_for_mandate(&self, mandate_id: Uuid) -> AtlasResult<i64>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DirectDebitDashboard>;
}

/// `PostgreSQL` stub implementation
#[allow(dead_code)]
pub struct PostgresDirectDebitMandateRepository { #[allow(dead_code)] pool: PgPool }
impl PostgresDirectDebitMandateRepository { #[must_use] 
pub const fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl DirectDebitMandateRepository for PostgresDirectDebitMandateRepository {
    async fn create_mandate(&self, _: Uuid, _: &str, _: Uuid, _: Option<&str>, _: Option<&str>, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: chrono::NaiveDate, _: Option<chrono::NaiveDate>, _: Option<&str>, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<DirectDebitMandate> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_mandate(&self, _: Uuid) -> AtlasResult<Option<DirectDebitMandate>> { Ok(None) }
    async fn get_mandate_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<DirectDebitMandate>> { Ok(None) }
    async fn list_mandates(&self, _: Uuid, _: Option<Uuid>, _: Option<&str>) -> AtlasResult<Vec<DirectDebitMandate>> { Ok(vec![]) }
    async fn update_mandate_status(&self, _: Uuid, _: &str, _: Option<&str>) -> AtlasResult<DirectDebitMandate> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_mandate_activation(&self, _: Uuid, _: chrono::NaiveDate) -> AtlasResult<DirectDebitMandate> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_mandate_collection_stats(&self, _: Uuid, _: i32, _: &str, _: Option<chrono::NaiveDate>, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn update_mandate_expiry(&self, _: Uuid, _: chrono::NaiveDate) -> AtlasResult<DirectDebitMandate> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn create_collection(&self, _: Uuid, _: Uuid, _: &str, _: &str, _: &str, _: &str, _: Option<Uuid>, _: Option<&str>, _: Option<Uuid>, _: chrono::NaiveDate, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<MandateCollection> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_collection(&self, _: Uuid) -> AtlasResult<Option<MandateCollection>> { Ok(None) }
    async fn list_collections(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<MandateCollection>> { Ok(vec![]) }
    async fn update_collection_status(&self, _: Uuid, _: &str, _: Option<&str>) -> AtlasResult<MandateCollection> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_collection_return(&self, _: Uuid, _: &str, _: Option<&str>) -> AtlasResult<MandateCollection> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn sum_collected_for_mandate(&self, _: Uuid) -> AtlasResult<String> { Ok("0.00".into()) }
    async fn count_collections_for_mandate(&self, _: Uuid) -> AtlasResult<i64> { Ok(0) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<DirectDebitDashboard> {
        Ok(DirectDebitDashboard {
            total_mandates: 0, draft_mandates: 0, active_mandates: 0, used_mandates: 0,
            cancelled_mandates: 0, expired_mandates: 0, revoked_mandates: 0,
            total_collections: 0, completed_collections: 0, failed_collections: 0,
            pending_collections: 0, total_collected_amount: "0.00".into(),
            pending_amount: "0.00".into(),
        })
    }
}
