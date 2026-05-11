//! Receivables Factoring Module
//!
//! Oracle Fusion Cloud ERP-inspired Receivables Factoring for Treasury.
//! Manages the sale of accounts receivable to third-party factors at a discount
//! for immediate cash. Supports recourse and non-recourse factoring with
//! advance tracking, fee calculations, and settlement management.
//!
//! Features:
//! - Factor Company management (financial institutions that purchase receivables)
//! - Factoring Agreement lifecycle (draft → active → suspended → terminated)
//! - Factoring Request workflow (draft → submitted → approved → funded → settled)
//! - Request lines for individual receivables being factored
//! - Settlement processing when customers pay the factor
//! - Numeric helpers for precise financial calculations
//! - Dashboard with factoring summary and metrics
//!
//! Oracle Fusion equivalent: Financials > Treasury > Receivables Factoring

mod engine;
pub mod numeric_helpers;

pub use engine::ReceivablesFactoringEngine;

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

// ============================================================================
// Constants
// ============================================================================

const VALID_RECOURSE_TYPES: &[&str] = &["recourse", "non_recourse"];
const VALID_AGREEMENT_TYPES: &[&str] = &["spot", "bulk", "maturity", "undisclosed"];
const VALID_AGREEMENT_STATUSES: &[&str] = &["draft", "active", "suspended", "expired", "terminated"];
const VALID_REQUEST_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "funded",
    "partially_settled", "settled", "cancelled", "rejected",
];
#[allow(dead_code)]
const VALID_LINE_STATUSES: &[&str] = &["pending", "funded", "settled", "chargeback", "excluded"];
const VALID_SETTLEMENT_STATUSES: &[&str] = &["draft", "processed", "cancelled"];

// ============================================================================
// Types
// ============================================================================

/// Factor Company — a financial institution that purchases receivables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorCompany {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub bank_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub default_advance_rate: String,
    pub default_fee_rate: String,
    pub default_recourse_type: String,
    pub minimum_invoice_amount: Option<String>,
    pub maximum_invoice_amount: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Factoring Agreement — master contract with a factor company
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoringAgreement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub agreement_number: String,
    pub factor_company_id: Uuid,
    pub factor_company_code: Option<String>,
    pub agreement_name: String,
    pub description: Option<String>,
    pub agreement_type: String,
    pub recourse_type: String,
    pub advance_rate: String,
    pub factoring_fee_rate: String,
    pub late_fee_rate: String,
    pub reserve_rate: String,
    pub minimum_fee: String,
    pub currency_code: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub credit_limit: Option<String>,
    pub total_factored_amount: String,
    pub total_advance_amount: String,
    pub total_fee_amount: String,
    pub total_reserve_amount: String,
    pub total_settled_amount: String,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Factoring Request — a request to factor a batch of receivables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoringRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_number: String,
    pub agreement_id: Uuid,
    pub agreement_number: Option<String>,
    pub factor_company_id: Option<Uuid>,
    pub factor_company_name: Option<String>,
    pub request_date: chrono::NaiveDate,
    pub funding_date: Option<chrono::NaiveDate>,
    pub settlement_date: Option<chrono::NaiveDate>,
    pub total_invoice_amount: String,
    pub eligible_amount: String,
    pub advance_rate: String,
    pub advance_amount: String,
    pub factoring_fee_rate: String,
    pub factoring_fee_amount: String,
    pub reserve_rate: String,
    pub reserve_amount: String,
    pub recourse_type: String,
    pub currency_code: String,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub funded_by: Option<Uuid>,
    pub funded_at: Option<chrono::DateTime<chrono::Utc>>,
    pub settled_by: Option<Uuid>,
    pub settled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Factoring Request Line — an individual receivable being factored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoringRequestLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_id: Uuid,
    pub line_number: i32,
    pub transaction_id: Option<Uuid>,
    pub transaction_number: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub invoice_due_date: Option<chrono::NaiveDate>,
    pub invoice_amount: String,
    pub eligible_amount: String,
    pub days_outstanding: Option<i32>,
    pub days_overdue: Option<i32>,
    pub advance_amount: String,
    pub factoring_fee_amount: String,
    pub reserve_amount: String,
    pub settlement_amount: String,
    pub is_eligible: bool,
    pub exclusion_reason: Option<String>,
    pub status: String,
    pub settled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Factoring Settlement — when customers pay the factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoringSettlement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub settlement_number: String,
    pub agreement_id: Uuid,
    pub request_id: Option<Uuid>,
    pub settlement_date: chrono::NaiveDate,
    pub total_settled: String,
    pub total_reserve_released: String,
    pub total_chargebacks: String,
    pub total_late_fees: String,
    pub net_to_customer: String,
    pub currency_code: String,
    pub status: String,
    pub processed_by: Option<Uuid>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Factoring Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoringDashboard {
    pub organization_id: Uuid,
    pub total_factor_companies: i64,
    pub active_factor_companies: i64,
    pub total_agreements: i64,
    pub active_agreements: i64,
    pub total_requests: i64,
    pub draft_requests: i64,
    pub approved_requests: i64,
    pub funded_requests: i64,
    pub settled_requests: i64,
    pub total_invoices_factored: String,
    pub total_advances: String,
    pub total_fees: String,
    pub total_reserves: String,
    pub total_settled: String,
}

// ============================================================================
// Repository Trait
// ============================================================================

#[async_trait]
pub trait ReceivablesFactoringRepository: Send + Sync {
    // Factor Companies
    async fn create_factor_company(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        contact_name: Option<&str>, contact_email: Option<&str>, contact_phone: Option<&str>,
        bank_name: Option<&str>, bank_account_number: Option<&str>,
        advance_rate: &str, fee_rate: &str, recourse_type: &str,
        minimum_invoice_amount: Option<&str>, maximum_invoice_amount: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FactorCompany>;

    async fn get_factor_company(&self, id: Uuid) -> AtlasResult<Option<FactorCompany>>;
    async fn get_factor_company_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FactorCompany>>;
    async fn list_factor_companies(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<FactorCompany>>;
    async fn deactivate_factor_company(&self, id: Uuid) -> AtlasResult<FactorCompany>;
    async fn activate_factor_company(&self, id: Uuid) -> AtlasResult<FactorCompany>;

    // Agreements
    async fn create_agreement(
        &self, org_id: Uuid, agreement_number: &str, factor_company_id: Uuid,
        agreement_name: &str, description: Option<&str>,
        agreement_type: &str, recourse_type: &str,
        advance_rate: &str, factoring_fee_rate: &str, late_fee_rate: &str,
        reserve_rate: &str, minimum_fee: &str, currency_code: &str,
        start_date: chrono::NaiveDate, end_date: Option<chrono::NaiveDate>,
        credit_limit: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<FactoringAgreement>;

    async fn get_agreement(&self, id: Uuid) -> AtlasResult<Option<FactoringAgreement>>;
    async fn list_agreements(&self, org_id: Uuid, status: Option<&str>, factor_company_id: Option<Uuid>) -> AtlasResult<Vec<FactoringAgreement>>;
    async fn update_agreement_status(&self, id: Uuid, status: &str, approved_by: Option<Uuid>) -> AtlasResult<FactoringAgreement>;

    // Requests
    async fn create_request(
        &self, org_id: Uuid, request_number: &str, agreement_id: Uuid,
        request_date: chrono::NaiveDate, recourse_type: &str,
        currency_code: &str, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<FactoringRequest>;

    async fn get_request(&self, id: Uuid) -> AtlasResult<Option<FactoringRequest>>;
    async fn list_requests(&self, org_id: Uuid, agreement_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<FactoringRequest>>;
    async fn update_request_status(&self, id: Uuid, status: &str) -> AtlasResult<FactoringRequest>;
    async fn update_request_funding(&self, id: Uuid, funded_by: Option<Uuid>) -> AtlasResult<FactoringRequest>;
    async fn update_request_settlement(&self, id: Uuid, settled_by: Option<Uuid>) -> AtlasResult<FactoringRequest>;

    // Request Lines
    async fn add_request_line(
        &self, org_id: Uuid, request_id: Uuid, line_number: i32,
        transaction_id: Option<Uuid>, transaction_number: Option<&str>,
        customer_id: Option<Uuid>, customer_number: Option<&str>, customer_name: Option<&str>,
        invoice_date: Option<chrono::NaiveDate>, invoice_due_date: Option<chrono::NaiveDate>,
        invoice_amount: &str, eligible_amount: &str,
        days_outstanding: Option<i32>, days_overdue: Option<i32>,
    ) -> AtlasResult<FactoringRequestLine>;

    async fn list_request_lines(&self, request_id: Uuid) -> AtlasResult<Vec<FactoringRequestLine>>;

    // Settlements
    async fn create_settlement(
        &self, org_id: Uuid, settlement_number: &str, agreement_id: Uuid,
        request_id: Option<Uuid>, settlement_date: chrono::NaiveDate,
        currency_code: &str, notes: Option<&str>, created_by: Option<Uuid>,
    ) -> AtlasResult<FactoringSettlement>;

    async fn list_settlements(&self, org_id: Uuid, agreement_id: Option<Uuid>, status: Option<&str>) -> AtlasResult<Vec<FactoringSettlement>>;
    async fn update_settlement_status(&self, id: Uuid, status: &str, processed_by: Option<Uuid>) -> AtlasResult<FactoringSettlement>;

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<FactoringDashboard>;
}

// ============================================================================
// PostgreSQL Implementation (Stub)
// ============================================================================

/// `PostgreSQL` implementation of `ReceivablesFactoringRepository`
#[allow(dead_code)]
pub struct PostgresReceivablesFactoringRepository {
    pool: PgPool,
}

impl PostgresReceivablesFactoringRepository {
    #[must_use] 
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ReceivablesFactoringRepository for PostgresReceivablesFactoringRepository {
    // Factor Companies
    async fn create_factor_company(
        &self, _: Uuid, _: &str, _: &str, _: Option<&str>,
        _: Option<&str>, _: Option<&str>, _: Option<&str>,
        _: Option<&str>, _: Option<&str>,
        _: &str, _: &str, _: &str,
        _: Option<&str>, _: Option<&str>,
        _: Option<Uuid>,
    ) -> AtlasResult<FactorCompany> {
        Err(AtlasError::DatabaseError("Not implemented".into()))
    }

    async fn get_factor_company(&self, _: Uuid) -> AtlasResult<Option<FactorCompany>> { Ok(None) }
    async fn get_factor_company_by_code(&self, _: Uuid, _: &str) -> AtlasResult<Option<FactorCompany>> { Ok(None) }
    async fn list_factor_companies(&self, _: Uuid, _: Option<bool>) -> AtlasResult<Vec<FactorCompany>> { Ok(vec![]) }
    async fn deactivate_factor_company(&self, _: Uuid) -> AtlasResult<FactorCompany> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn activate_factor_company(&self, _: Uuid) -> AtlasResult<FactorCompany> { Err(AtlasError::EntityNotFound("Not found".into())) }

    // Agreements
    async fn create_agreement(
        &self, _: Uuid, _: &str, _: Uuid, _: &str, _: Option<&str>,
        _: &str, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str, _: &str,
        _: chrono::NaiveDate, _: Option<chrono::NaiveDate>, _: Option<&str>, _: Option<Uuid>,
    ) -> AtlasResult<FactoringAgreement> {
        Err(AtlasError::DatabaseError("Not implemented".into()))
    }

    async fn get_agreement(&self, _: Uuid) -> AtlasResult<Option<FactoringAgreement>> { Ok(None) }
    async fn list_agreements(&self, _: Uuid, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<Vec<FactoringAgreement>> { Ok(vec![]) }
    async fn update_agreement_status(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<FactoringAgreement> { Err(AtlasError::EntityNotFound("Not found".into())) }

    // Requests
    async fn create_request(
        &self, _: Uuid, _: &str, _: Uuid, _: chrono::NaiveDate, _: &str, _: &str, _: Option<&str>, _: Option<Uuid>,
    ) -> AtlasResult<FactoringRequest> {
        Err(AtlasError::DatabaseError("Not implemented".into()))
    }

    async fn get_request(&self, _: Uuid) -> AtlasResult<Option<FactoringRequest>> { Ok(None) }
    async fn list_requests(&self, _: Uuid, _: Option<Uuid>, _: Option<&str>) -> AtlasResult<Vec<FactoringRequest>> { Ok(vec![]) }
    async fn update_request_status(&self, _: Uuid, _: &str) -> AtlasResult<FactoringRequest> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_request_funding(&self, _: Uuid, _: Option<Uuid>) -> AtlasResult<FactoringRequest> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_request_settlement(&self, _: Uuid, _: Option<Uuid>) -> AtlasResult<FactoringRequest> { Err(AtlasError::EntityNotFound("Not found".into())) }

    // Request Lines
    async fn add_request_line(
        &self, _: Uuid, _: Uuid, _: i32, _: Option<Uuid>, _: Option<&str>,
        _: Option<Uuid>, _: Option<&str>, _: Option<&str>,
        _: Option<chrono::NaiveDate>, _: Option<chrono::NaiveDate>,
        _: &str, _: &str, _: Option<i32>, _: Option<i32>,
    ) -> AtlasResult<FactoringRequestLine> {
        Err(AtlasError::DatabaseError("Not implemented".into()))
    }

    async fn list_request_lines(&self, _: Uuid) -> AtlasResult<Vec<FactoringRequestLine>> { Ok(vec![]) }

    // Settlements
    async fn create_settlement(
        &self, _: Uuid, _: &str, _: Uuid, _: Option<Uuid>, _: chrono::NaiveDate, _: &str, _: Option<&str>, _: Option<Uuid>,
    ) -> AtlasResult<FactoringSettlement> {
        Err(AtlasError::DatabaseError("Not implemented".into()))
    }

    async fn list_settlements(&self, _: Uuid, _: Option<Uuid>, _: Option<&str>) -> AtlasResult<Vec<FactoringSettlement>> { Ok(vec![]) }
    async fn update_settlement_status(&self, _: Uuid, _: &str, _: Option<Uuid>) -> AtlasResult<FactoringSettlement> { Err(AtlasError::EntityNotFound("Not found".into())) }

    // Dashboard
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<FactoringDashboard> {
        Ok(FactoringDashboard {
            organization_id: org_id,
            total_factor_companies: 0,
            active_factor_companies: 0,
            total_agreements: 0,
            active_agreements: 0,
            total_requests: 0,
            draft_requests: 0,
            approved_requests: 0,
            funded_requests: 0,
            settled_requests: 0,
            total_invoices_factored: "0.00".into(),
            total_advances: "0.00".into(),
            total_fees: "0.00".into(),
            total_reserves: "0.00".into(),
            total_settled: "0.00".into(),
        })
    }
}
