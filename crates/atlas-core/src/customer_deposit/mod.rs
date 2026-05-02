//! Customer Deposit Module
//!
//! Oracle Fusion Cloud ERP-inspired Customer Deposit management.
//! Manages customer advance deposits, application to invoices, and refunds.
//!
//! Features:
//! - Customer deposit creation and receipt
//! - Application of deposits to AR invoices
//! - Partial/full application tracking
//! - Unapplied deposit management
//! - Refund processing for unapplied deposits
//!
//! Oracle Fusion equivalent: Financials > Accounts Receivable > Customer Deposits

mod engine;

pub use engine::CustomerDepositEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Customer deposit header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerDeposit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub deposit_number: String,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub customer_site_id: Option<Uuid>,
    pub description: Option<String>,
    /// 'draft', 'received', 'partially_applied', 'fully_applied', 'refunded', 'cancelled'
    pub status: String,
    pub currency_code: String,
    pub deposit_amount: String,
    pub applied_amount: String,
    pub unapplied_amount: String,
    pub exchange_rate: Option<String>,
    /// GL account for the deposit (customer deposit liability)
    pub deposit_account_code: Option<String>,
    /// GL account for the receivable
    pub receivable_account_code: Option<String>,
    pub deposit_date: chrono::NaiveDate,
    pub receipt_date: Option<chrono::NaiveDate>,
    pub receipt_reference: Option<String>,
    pub expiration_date: Option<chrono::NaiveDate>,
    pub received_by: Option<Uuid>,
    pub received_at: Option<chrono::DateTime<chrono::Utc>>,
    pub refund_reference: Option<String>,
    pub refunded_by: Option<Uuid>,
    pub refunded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancelled_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Application of a deposit to an invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositApplication {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub deposit_id: Uuid,
    pub deposit_number: Option<String>,
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

/// Customer deposit dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerDepositDashboard {
    pub total_deposits: i32,
    pub draft_deposits: i32,
    pub open_deposits: i32,
    pub total_deposit_amount: String,
    pub total_applied_amount: String,
    pub total_unapplied_amount: String,
    pub total_refunded_amount: String,
    pub deposits_by_customer: serde_json::Value,
    pub aging_buckets: serde_json::Value,
}

/// Repository trait
#[async_trait]
pub trait CustomerDepositRepository: Send + Sync {
    async fn create_deposit(&self, org_id: Uuid, deposit_number: &str, customer_id: Uuid, customer_name: &str, customer_site_id: Option<Uuid>, description: Option<&str>, currency_code: &str, deposit_amount: &str, exchange_rate: Option<&str>, deposit_account_code: Option<&str>, receivable_account_code: Option<&str>, deposit_date: chrono::NaiveDate, expiration_date: Option<chrono::NaiveDate>, created_by: Option<Uuid>) -> AtlasResult<CustomerDeposit>;
    async fn get_deposit(&self, id: Uuid) -> AtlasResult<Option<CustomerDeposit>>;
    async fn get_deposit_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<CustomerDeposit>>;
    async fn list_deposits(&self, org_id: Uuid, status: Option<&str>, customer_id: Option<Uuid>) -> AtlasResult<Vec<CustomerDeposit>>;
    async fn update_deposit_status(&self, id: Uuid, status: &str) -> AtlasResult<CustomerDeposit>;
    async fn update_deposit_amounts(&self, id: Uuid, applied: &str, unapplied: &str, status: &str) -> AtlasResult<()>;
    async fn update_receipt_info(&self, id: Uuid, receipt_ref: Option<&str>, received_by: Option<Uuid>) -> AtlasResult<CustomerDeposit>;
    async fn update_refund_info(&self, id: Uuid, refund_ref: Option<&str>, refunded_by: Option<Uuid>) -> AtlasResult<CustomerDeposit>;
    async fn create_application(&self, org_id: Uuid, deposit_id: Uuid, deposit_number: Option<&str>, invoice_id: Uuid, invoice_number: Option<&str>, applied_amount: &str, application_date: chrono::NaiveDate, gl_account_code: Option<&str>, applied_by: Option<Uuid>) -> AtlasResult<DepositApplication>;
    async fn get_application(&self, id: Uuid) -> AtlasResult<Option<DepositApplication>>;
    async fn list_applications_by_deposit(&self, deposit_id: Uuid) -> AtlasResult<Vec<DepositApplication>>;
    async fn list_applications_by_invoice(&self, invoice_id: Uuid) -> AtlasResult<Vec<DepositApplication>>;
    async fn update_application_status(&self, id: Uuid, status: &str, reversed_by: Option<Uuid>, reversal_reason: Option<&str>) -> AtlasResult<DepositApplication>;
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<CustomerDepositDashboard>;
}

/// PostgreSQL implementation (stub)
pub struct PostgresCustomerDepositRepository { pool: PgPool }
impl PostgresCustomerDepositRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl CustomerDepositRepository for PostgresCustomerDepositRepository {
    async fn create_deposit(&self, _: Uuid, _: &str, _: Uuid, _: &str, _: Option<Uuid>, _: Option<&str>, _: &str, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>, _: chrono::NaiveDate, _: Option<chrono::NaiveDate>, _: Option<Uuid>) -> AtlasResult<CustomerDeposit> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_deposit(&self, _: Uuid) -> AtlasResult<Option<CustomerDeposit>> { Ok(None) }
    async fn get_deposit_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<CustomerDeposit>> { Ok(None) }
    async fn list_deposits(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<Vec<CustomerDeposit>> { Ok(vec![]) }
    async fn update_deposit_status(&self, _: Uuid, _: &str) -> AtlasResult<CustomerDeposit> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_deposit_amounts(&self, _: Uuid, _: &str, _: &str, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn update_receipt_info(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<CustomerDeposit> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn update_refund_info(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<CustomerDeposit> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn create_application(&self, _: Uuid, _: Uuid, _: Option<&str>, _: Uuid, _: Option<&str>, _: &str, _: chrono::NaiveDate, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<DepositApplication> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_application(&self, _: Uuid) -> AtlasResult<Option<DepositApplication>> { Ok(None) }
    async fn list_applications_by_deposit(&self, _: Uuid) -> AtlasResult<Vec<DepositApplication>> { Ok(vec![]) }
    async fn list_applications_by_invoice(&self, _: Uuid) -> AtlasResult<Vec<DepositApplication>> { Ok(vec![]) }
    async fn update_application_status(&self, _: Uuid, _: &str, _: Option<Uuid>, _: Option<&str>) -> AtlasResult<DepositApplication> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<CustomerDepositDashboard> { Ok(CustomerDepositDashboard { total_deposits: 0, draft_deposits: 0, open_deposits: 0, total_deposit_amount: "0".into(), total_applied_amount: "0".into(), total_unapplied_amount: "0".into(), total_refunded_amount: "0".into(), deposits_by_customer: serde_json::json!([]), aging_buckets: serde_json::json!([]) }) }
}
