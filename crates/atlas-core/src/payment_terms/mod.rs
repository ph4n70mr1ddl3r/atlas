//! Payment Terms Module
//!
//! Oracle Fusion: Financials > Payment Terms Management
//! Defines payment terms with discount schedules for AP and AR.

mod engine;
pub use engine::PaymentTermsEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTerm {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub term_code: String,
    pub name: String,
    pub description: Option<String>,
    pub base_due_days: i32,
    pub due_date_cutoff_day: Option<i32>,
    pub status: String,
    pub term_type: String,
    pub default_discount_percent: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTermDiscountSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payment_term_id: Uuid,
    pub discount_percent: String,
    pub discount_days: i32,
    pub discount_day_of_month: Option<i32>,
    pub discount_basis: String,
    pub display_order: i32,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTermInstallment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payment_term_id: Uuid,
    pub installment_number: i32,
    pub due_days_offset: i32,
    pub percentage: String,
    pub discount_percent: String,
    pub discount_days: i32,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTermDashboard {
    pub total_terms: i32,
    pub active_terms: i32,
    pub terms_with_discounts: i32,
    pub terms_with_installments: i32,
}

#[async_trait]
pub trait PaymentTermsRepository: Send + Sync {
    async fn create_term(&self, org_id: Uuid, term_code: &str, name: &str, description: Option<&str>, base_due_days: i32, due_date_cutoff_day: Option<i32>, term_type: &str, default_discount_percent: &str, created_by: Option<Uuid>) -> AtlasResult<PaymentTerm>;
    async fn get_term(&self, org_id: Uuid, term_code: &str) -> AtlasResult<Option<PaymentTerm>>;
    async fn get_term_by_id(&self, id: Uuid) -> AtlasResult<Option<PaymentTerm>>;
    async fn list_terms(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PaymentTerm>>;
    async fn update_term_status(&self, id: Uuid, status: &str) -> AtlasResult<PaymentTerm>;
    async fn delete_term(&self, id: Uuid) -> AtlasResult<()>;

    async fn create_discount_schedule(&self, org_id: Uuid, term_id: Uuid, discount_percent: &str, discount_days: i32, discount_day_of_month: Option<i32>, discount_basis: &str, display_order: i32) -> AtlasResult<PaymentTermDiscountSchedule>;
    async fn list_discount_schedules(&self, term_id: Uuid) -> AtlasResult<Vec<PaymentTermDiscountSchedule>>;
    async fn delete_discount_schedule(&self, id: Uuid) -> AtlasResult<()>;

    async fn create_installment(&self, org_id: Uuid, term_id: Uuid, installment_number: i32, due_days_offset: i32, percentage: &str, discount_percent: &str, discount_days: i32) -> AtlasResult<PaymentTermInstallment>;
    async fn list_installments(&self, term_id: Uuid) -> AtlasResult<Vec<PaymentTermInstallment>>;
    async fn delete_installment(&self, id: Uuid) -> AtlasResult<()>;

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PaymentTermDashboard>;
}

pub struct PostgresPaymentTermsRepository { pool: PgPool }
impl PostgresPaymentTermsRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl PaymentTermsRepository for PostgresPaymentTermsRepository {
    async fn create_term(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: i32, _: Option<i32>, _: &str, _: &str, _: Option<Uuid>) -> AtlasResult<PaymentTerm> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_term(&self, _: Uuid, _: &str) -> AtlasResult<Option<PaymentTerm>> { Ok(None) }
    async fn get_term_by_id(&self, _: Uuid) -> AtlasResult<Option<PaymentTerm>> { Ok(None) }
    async fn list_terms(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<PaymentTerm>> { Ok(vec![]) }
    async fn update_term_status(&self, _: Uuid, _: &str) -> AtlasResult<PaymentTerm> { Err(AtlasError::EntityNotFound("Mock".into())) }
    async fn delete_term(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn create_discount_schedule(&self, _: Uuid, _: Uuid, _: &str, _: i32, _: Option<i32>, _: &str, _: i32) -> AtlasResult<PaymentTermDiscountSchedule> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_discount_schedules(&self, _: Uuid) -> AtlasResult<Vec<PaymentTermDiscountSchedule>> { Ok(vec![]) }
    async fn delete_discount_schedule(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn create_installment(&self, _: Uuid, _: Uuid, _: i32, _: i32, _: &str, _: &str, _: i32) -> AtlasResult<PaymentTermInstallment> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_installments(&self, _: Uuid) -> AtlasResult<Vec<PaymentTermInstallment>> { Ok(vec![]) }
    async fn delete_installment(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn get_dashboard(&self, _: Uuid) -> AtlasResult<PaymentTermDashboard> { Ok(PaymentTermDashboard { total_terms: 0, active_terms: 0, terms_with_discounts: 0, terms_with_installments: 0 }) }
}
