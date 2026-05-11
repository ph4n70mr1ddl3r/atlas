//! Third-Party Payment Management Module
//!
//! Oracle Fusion Cloud ERP-inspired Third-Party Payments for Accounts Payable.
//! Manages payments made to third parties (tax authorities, garnishment recipients,
//! insurance providers, court-ordered deductions) on behalf of suppliers or employees.
//!
//! Features:
//! - Create third-party payment instructions with configurable types
//! - Link payments to source entities (suppliers, employees)
//! - Approval workflow: draft → submitted → approved → paid / cancelled
//! - Recurring third-party payment schedules
//! - Payment hold/release mechanism
//! - Dashboard with payment summaries and aging
//!
//! Oracle Fusion equivalent: Financials > Payables > Third-Party Payments

mod engine;

pub use engine::ThirdPartyPaymentEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

/// Third-party payment header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyPayment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payment_number: String,
    /// 'garnishment', '`tax_levy`', 'insurance', '`court_order`', 'custom'
    pub payment_type: String,
    /// 'draft', 'submitted', 'approved', 'paid', '`on_hold`', 'cancelled', 'rejected'
    pub status: String,
    /// The entity on whose behalf the payment is made (supplier or employee)
    pub source_entity_type: String,
    pub source_entity_id: Uuid,
    pub source_entity_name: String,
    /// The third-party recipient of the payment
    pub payee_name: String,
    pub payee_tax_id: Option<String>,
    pub payee_address: Option<String>,
    pub payee_bank_account: Option<String>,
    /// Payment details
    pub amount: String,
    pub currency_code: String,
    pub payment_method: Option<String>,
    pub payment_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub reference_document: Option<String>,
    pub reference_document_id: Option<Uuid>,
    pub description: Option<String>,
    pub case_number: Option<String>,
    pub court_jurisdiction: Option<String>,
    /// Recurrence
    pub is_recurring: bool,
    pub recurrence_frequency: Option<String>,
    pub recurrence_start_date: Option<chrono::NaiveDate>,
    pub recurrence_end_date: Option<chrono::NaiveDate>,
    /// Approval
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Hold
    pub hold_reason: Option<String>,
    pub hold_at: Option<chrono::DateTime<chrono::Utc>>,
    pub hold_by: Option<Uuid>,
    /// Payment confirmation
    pub payment_reference: Option<String>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub paid_by: Option<Uuid>,
    /// Cancellation
    pub cancellation_reason: Option<String>,
    pub cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cancelled_by: Option<Uuid>,
    /// Rejection
    pub rejection_reason: Option<String>,
    pub rejected_by: Option<Uuid>,
    pub rejected_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Audit
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Third-party payment line (breakdown of the payment amount)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyPaymentLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payment_id: Uuid,
    pub line_number: i32,
    pub line_type: String,
    pub description: Option<String>,
    pub amount: String,
    pub gl_account: Option<String>,
    pub cost_center: Option<String>,
    pub tax_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Third-party payment dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThirdPartyPaymentDashboard {
    pub total_payments: i64,
    pub draft_count: i64,
    pub pending_approval_count: i64,
    pub approved_count: i64,
    pub paid_count: i64,
    pub on_hold_count: i64,
    pub cancelled_count: i64,
    pub total_amount: String,
    pub paid_amount: String,
    pub pending_amount: String,
    pub on_hold_amount: String,
    pub payments_by_type: serde_json::Value,
    pub upcoming_due: serde_json::Value,
}

/// Repository trait for third-party payment persistence
#[async_trait]
pub trait ThirdPartyPaymentRepository: Send + Sync {
    async fn create_payment(
        &self, org_id: Uuid, payment_number: &str, payment_type: &str,
        source_entity_type: &str, source_entity_id: Uuid, source_entity_name: &str,
        payee_name: &str, payee_tax_id: Option<&str>, payee_address: Option<&str>,
        payee_bank_account: Option<&str>, amount: &str, currency_code: &str,
        payment_method: Option<&str>, payment_date: Option<chrono::NaiveDate>,
        due_date: Option<chrono::NaiveDate>, reference_document: Option<&str>,
        reference_document_id: Option<Uuid>, description: Option<&str>,
        case_number: Option<&str>, court_jurisdiction: Option<&str>,
        is_recurring: bool, recurrence_frequency: Option<&str>,
        recurrence_start_date: Option<chrono::NaiveDate>,
        recurrence_end_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ThirdPartyPayment>;

    async fn get_payment(&self, id: Uuid) -> AtlasResult<Option<ThirdPartyPayment>>;
    async fn get_payment_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<ThirdPartyPayment>>;
    async fn list_payments(
        &self, org_id: Uuid, status: Option<&str>, payment_type: Option<&str>,
        source_entity_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ThirdPartyPayment>>;

    async fn update_status(&self, id: Uuid, status: &str) -> AtlasResult<ThirdPartyPayment>;
    async fn submit_payment(&self, id: Uuid) -> AtlasResult<ThirdPartyPayment>;
    async fn approve_payment(&self, id: Uuid, approved_by: Option<Uuid>) -> AtlasResult<ThirdPartyPayment>;
    async fn reject_payment(&self, id: Uuid, reason: Option<&str>, rejected_by: Option<Uuid>) -> AtlasResult<ThirdPartyPayment>;
    async fn place_on_hold(&self, id: Uuid, reason: Option<&str>, held_by: Option<Uuid>) -> AtlasResult<ThirdPartyPayment>;
    async fn release_hold(&self, id: Uuid) -> AtlasResult<ThirdPartyPayment>;
    async fn record_payment(&self, id: Uuid, payment_ref: &str, paid_by: Option<Uuid>) -> AtlasResult<ThirdPartyPayment>;
    async fn cancel_payment(&self, id: Uuid, reason: Option<&str>, cancelled_by: Option<Uuid>) -> AtlasResult<ThirdPartyPayment>;

    async fn add_line(
        &self, org_id: Uuid, payment_id: Uuid, line_number: i32,
        line_type: &str, description: Option<&str>, amount: &str,
        gl_account: Option<&str>, cost_center: Option<&str>,
        tax_code: Option<&str>,
    ) -> AtlasResult<ThirdPartyPaymentLine>;

    async fn list_lines(&self, payment_id: Uuid) -> AtlasResult<Vec<ThirdPartyPaymentLine>>;
    async fn get_line(&self, line_id: Uuid) -> AtlasResult<Option<ThirdPartyPaymentLine>>;
    async fn remove_line(&self, line_id: Uuid) -> AtlasResult<()>;

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ThirdPartyPaymentDashboard>;
}

/// `PostgreSQL` stub implementation
#[allow(dead_code)]
pub struct PostgresThirdPartyPaymentRepository { #[allow(dead_code)] pool: PgPool }
impl PostgresThirdPartyPaymentRepository { #[must_use] 
pub const fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl ThirdPartyPaymentRepository for PostgresThirdPartyPaymentRepository {
    async fn create_payment(
        &self, _: Uuid, _: &str, _: &str, _: &str, _: Uuid, _: &str,
        _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>,
        _: &str, _: &str, _: Option<&str>, _: Option<chrono::NaiveDate>,
        _: Option<chrono::NaiveDate>, _: Option<&str>, _: Option<Uuid>,
        _: Option<&str>, _: Option<&str>, _: Option<&str>,
        _: bool, _: Option<&str>, _: Option<chrono::NaiveDate>,
        _: Option<chrono::NaiveDate>, _: Option<Uuid>,
    ) -> AtlasResult<ThirdPartyPayment> { Err(AtlasError::DatabaseError("Not implemented".into())) }

    async fn get_payment(&self, _: Uuid) -> AtlasResult<Option<ThirdPartyPayment>> { Ok(None) }
    async fn get_payment_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<ThirdPartyPayment>> { Ok(None) }
    async fn list_payments(&self, _: Uuid, _: Option<&str>, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<Vec<ThirdPartyPayment>> { Ok(vec![]) }
    async fn update_status(&self, _: Uuid, _: &str) -> AtlasResult<ThirdPartyPayment> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn submit_payment(&self, _: Uuid) -> AtlasResult<ThirdPartyPayment> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn approve_payment(&self, _: Uuid, _: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn reject_payment(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn place_on_hold(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn release_hold(&self, _: Uuid) -> AtlasResult<ThirdPartyPayment> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn record_payment(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn cancel_payment(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<ThirdPartyPayment> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn add_line(&self, _: Uuid, _: Uuid, _: i32, _: &str, _: Option<&str>, _: &str, _: Option<&str>, _: Option<&str>, _: Option<&str>) -> AtlasResult<ThirdPartyPaymentLine> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_lines(&self, _: Uuid) -> AtlasResult<Vec<ThirdPartyPaymentLine>> { Ok(vec![]) }
    async fn get_line(&self, _: Uuid) -> AtlasResult<Option<ThirdPartyPaymentLine>> { Ok(None) }
    async fn remove_line(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<ThirdPartyPaymentDashboard> {
        Ok(ThirdPartyPaymentDashboard {
            total_payments: 0, draft_count: 0, pending_approval_count: 0,
            approved_count: 0, paid_count: 0, on_hold_count: 0, cancelled_count: 0,
            total_amount: "0.00".into(), paid_amount: "0.00".into(),
            pending_amount: "0.00".into(), on_hold_amount: "0.00".into(),
            payments_by_type: serde_json::json!([]), upcoming_due: serde_json::json!([]),
        })
    }
}
