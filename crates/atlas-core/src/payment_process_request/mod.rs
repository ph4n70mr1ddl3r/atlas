//! Payment Process Request (PPR) Module
//!
//! Oracle Fusion Cloud ERP-inspired Payment Process Request.
//! Automates batch payment processing for Accounts Payable:
//! - Create PPR with invoice selection criteria
//! - Auto-select invoices matching criteria (supplier, due date, amount, payment method)
//! - Review and modify selected invoices
//! - Build payments (group invoices by supplier into payments)
//! - Format payments for bank file generation
//! - Confirm payments and mark invoices as paid
//!
//! Oracle Fusion equivalent: Financials > Payables > Payment Process Requests
//!
//! Lifecycle:
//!   draft → submitted → built → formatted → confirmed → completed
//!                  ↘ rejected           ↘ cancelled

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, NaiveDate, Utc};
use std::sync::Arc;
use tracing::info;

// ============================================================================
// Constants
// ============================================================================

const VALID_STATUSES: &[&str] = &[
    "draft", "submitted", "built", "formatted", "confirmed", "completed",
    "cancelled", "rejected",
];

const VALID_PAYMENT_METHODS: &[&str] = &[
    "check", "electronic", "wire", "ach", "sepa", "manual",
];

const VALID_INVOICE_STATUSES_IN_PPR: &[&str] = &[
    "selected", "excluded", "paid", "error",
];

// ============================================================================
// Types
// ============================================================================

/// A Payment Process Request — the parent batch record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProcessRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ppr_number: String,
    pub description: Option<String>,
    /// Payment date (when payments should be issued)
    pub payment_date: NaiveDate,
    /// GL date for the payment accounting entries
    pub gl_date: NaiveDate,
    /// Currency of the batch
    pub currency_code: String,
    /// Payment method filter — only invoices payable via this method are selected
    pub payment_method: String,
    /// Bank account from which payments are drawn
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    /// Payment format code (e.g. "EFT_US", "CHECK_STD")
    pub payment_format_code: Option<String>,
    /// Selection criteria
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub from_due_date: Option<NaiveDate>,
    pub to_due_date: Option<NaiveDate>,
    pub min_amount: Option<String>,
    pub max_amount: Option<String>,
    pub include_discounted: bool,
    pub pay_only_due: bool,
    /// Aggregate amounts
    pub total_invoice_count: i32,
    pub total_invoice_amount: String,
    pub total_payment_count: i32,
    pub total_payment_amount: String,
    /// Status
    pub status: String,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub built_by: Option<Uuid>,
    pub built_at: Option<DateTime<Utc>>,
    pub confirmed_by: Option<Uuid>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub completed_by: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_reason: Option<String>,
    pub reject_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An invoice line selected for payment within a PPR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PprInvoiceLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ppr_id: Uuid,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub invoice_amount: String,
    pub amount_due: String,
    pub discount_available: String,
    pub payment_amount: String,
    pub due_date: Option<NaiveDate>,
    pub payment_method: Option<String>,
    /// selected, excluded, paid, error
    pub status: String,
    pub exclude_reason: Option<String>,
    pub payment_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A built payment within a PPR (one per supplier per PPR).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PprPayment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ppr_id: Uuid,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub supplier_site_id: Option<Uuid>,
    pub payment_number: String,
    pub payment_method: String,
    pub bank_account_id: Option<Uuid>,
    pub payment_amount: String,
    pub invoice_count: i32,
    /// pending, formatted, confirmed, voided, error
    pub status: String,
    pub formatted_at: Option<DateTime<Utc>>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub document_number: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dashboard summary for PPR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PprDashboard {
    pub organization_id: Uuid,
    pub total_pprs: i32,
    pub draft_pprs: i32,
    pub pending_pprs: i32,
    pub completed_pprs: i32,
    pub total_payments_made: i32,
    pub total_amount_paid: String,
    pub by_payment_method: serde_json::Value,
}

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait PprRepository: Send + Sync {
    // ── PPR ──
    async fn create_ppr(
        &self,
        org_id: Uuid, ppr_number: &str, description: Option<&str>,
        payment_date: NaiveDate, gl_date: NaiveDate,
        currency_code: &str, payment_method: &str,
        bank_account_id: Option<Uuid>, bank_account_name: Option<&str>,
        payment_format_code: Option<&str>,
        supplier_id: Option<Uuid>, supplier_name: Option<&str>,
        from_due_date: Option<NaiveDate>, to_due_date: Option<NaiveDate>,
        min_amount: Option<&str>, max_amount: Option<&str>,
        include_discounted: bool, pay_only_due: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentProcessRequest>;

    async fn get_ppr(&self, id: Uuid) -> AtlasResult<Option<PaymentProcessRequest>>;
    async fn get_ppr_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<PaymentProcessRequest>>;
    async fn list_pprs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PaymentProcessRequest>>;
    async fn update_ppr_status(&self, id: Uuid, status: &str) -> AtlasResult<PaymentProcessRequest>;
    async fn update_ppr_aggregates(
        &self, id: Uuid,
        invoice_count: i32, invoice_amount: &str,
        payment_count: i32, payment_amount: &str,
    ) -> AtlasResult<()>;
    async fn update_ppr_submission(&self, id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest>;
    async fn update_ppr_built(&self, id: Uuid, built_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest>;
    async fn update_ppr_confirmed(&self, id: Uuid, confirmed_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest>;
    async fn update_ppr_completed(&self, id: Uuid, completed_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest>;
    async fn update_ppr_cancel_reason(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<PaymentProcessRequest>;
    async fn update_ppr_reject_reason(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<PaymentProcessRequest>;

    // ── Invoice lines ──
    async fn add_invoice_line(
        &self,
        org_id: Uuid, ppr_id: Uuid, invoice_id: Uuid,
        invoice_number: Option<&str>, supplier_id: Option<Uuid>,
        supplier_name: Option<&str>, invoice_amount: &str,
        amount_due: &str, discount_available: &str,
        payment_amount: &str, due_date: Option<NaiveDate>,
        payment_method: Option<&str>,
    ) -> AtlasResult<PprInvoiceLine>;

    async fn get_invoice_line(&self, id: Uuid) -> AtlasResult<Option<PprInvoiceLine>>;
    async fn list_invoice_lines(&self, ppr_id: Uuid) -> AtlasResult<Vec<PprInvoiceLine>>;
    async fn update_invoice_line_status(&self, id: Uuid, status: &str, reason: Option<&str>) -> AtlasResult<PprInvoiceLine>;
    async fn update_invoice_line_payment(&self, id: Uuid, payment_id: Uuid) -> AtlasResult<PprInvoiceLine>;
    async fn remove_invoice_lines_by_ppr(&self, ppr_id: Uuid) -> AtlasResult<()>;

    // ── Payments ──
    async fn create_payment(
        &self,
        org_id: Uuid, ppr_id: Uuid, supplier_id: Uuid,
        supplier_name: &str, supplier_site_id: Option<Uuid>,
        payment_number: &str, payment_method: &str,
        bank_account_id: Option<Uuid>, payment_amount: &str,
        invoice_count: i32,
    ) -> AtlasResult<PprPayment>;

    async fn get_payment(&self, id: Uuid) -> AtlasResult<Option<PprPayment>>;
    async fn list_payments(&self, ppr_id: Uuid) -> AtlasResult<Vec<PprPayment>>;
    async fn update_payment_status(&self, id: Uuid, status: &str) -> AtlasResult<PprPayment>;
    async fn update_payment_formatted(&self, id: Uuid, doc_number: Option<&str>) -> AtlasResult<PprPayment>;
    async fn remove_payments_by_ppr(&self, ppr_id: Uuid) -> AtlasResult<()>;

    // ── Dashboard ──
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PprDashboard>;
}

/// PostgreSQL stub implementation
#[allow(dead_code)]
pub struct PostgresPprRepository { #[allow(dead_code)] pool: PgPool }
impl PostgresPprRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl PprRepository for PostgresPprRepository {
    async fn create_ppr(&self, _: Uuid, _: &str, _: Option<&str>, _: NaiveDate, _: NaiveDate, _: &str, _: &str, _: Option<Uuid>, _: Option<&str>, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: Option<NaiveDate>, _: Option<NaiveDate>, _: Option<&str>, _: Option<&str>, _: bool, _: bool, _: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_ppr(&self, _: Uuid) -> AtlasResult<Option<PaymentProcessRequest>> { Ok(None) }
    async fn get_ppr_by_number(&self, _: Uuid, _: &str) -> AtlasResult<Option<PaymentProcessRequest>> { Ok(None) }
    async fn list_pprs(&self, _: Uuid, _: Option<&str>) -> AtlasResult<Vec<PaymentProcessRequest>> { Ok(vec![]) }
    async fn update_ppr_status(&self, _: Uuid, _: &str) -> AtlasResult<PaymentProcessRequest> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_ppr_aggregates(&self, _: Uuid, _: i32, _: &str, _: i32, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn update_ppr_submission(&self, _: Uuid, _: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_ppr_built(&self, _: Uuid, _: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_ppr_confirmed(&self, _: Uuid, _: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_ppr_completed(&self, _: Uuid, _: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_ppr_cancel_reason(&self, _: Uuid, _: Option<&str>) -> AtlasResult<PaymentProcessRequest> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_ppr_reject_reason(&self, _: Uuid, _: Option<&str>) -> AtlasResult<PaymentProcessRequest> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn add_invoice_line(&self, _: Uuid, _: Uuid, _: Uuid, _: Option<&str>, _: Option<Uuid>, _: Option<&str>, _: &str, _: &str, _: &str, _: &str, _: Option<NaiveDate>, _: Option<&str>) -> AtlasResult<PprInvoiceLine> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_invoice_line(&self, _: Uuid) -> AtlasResult<Option<PprInvoiceLine>> { Ok(None) }
    async fn list_invoice_lines(&self, _: Uuid) -> AtlasResult<Vec<PprInvoiceLine>> { Ok(vec![]) }
    async fn update_invoice_line_status(&self, _: Uuid, _: &str, _: Option<&str>) -> AtlasResult<PprInvoiceLine> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_invoice_line_payment(&self, _: Uuid, _: Uuid) -> AtlasResult<PprInvoiceLine> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn remove_invoice_lines_by_ppr(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn create_payment(&self, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Option<Uuid>, _: &str, _: &str, _: Option<Uuid>, _: &str, _: i32) -> AtlasResult<PprPayment> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_payment(&self, _: Uuid) -> AtlasResult<Option<PprPayment>> { Ok(None) }
    async fn list_payments(&self, _: Uuid) -> AtlasResult<Vec<PprPayment>> { Ok(vec![]) }
    async fn update_payment_status(&self, _: Uuid, _: &str) -> AtlasResult<PprPayment> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn update_payment_formatted(&self, _: Uuid, _: Option<&str>) -> AtlasResult<PprPayment> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn remove_payments_by_ppr(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PprDashboard> {
        Ok(PprDashboard {
            organization_id: org_id, total_pprs: 0, draft_pprs: 0, pending_pprs: 0,
            completed_pprs: 0, total_payments_made: 0, total_amount_paid: "0".into(),
            by_payment_method: serde_json::json!([]),
        })
    }
}

// ============================================================================
// Engine
// ============================================================================

pub struct PaymentProcessRequestEngine {
    repository: Arc<dyn PprRepository>,
}

impl PaymentProcessRequestEngine {
    pub fn new(repository: Arc<dyn PprRepository>) -> Self {
        Self { repository }
    }

    // ── PPR CRUD ──

    /// Create a new Payment Process Request in draft status.
    pub async fn create(
        &self,
        org_id: Uuid,
        description: Option<&str>,
        payment_date: NaiveDate,
        gl_date: NaiveDate,
        currency_code: &str,
        payment_method: &str,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        payment_format_code: Option<&str>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        from_due_date: Option<NaiveDate>,
        to_due_date: Option<NaiveDate>,
        min_amount: Option<&str>,
        max_amount: Option<&str>,
        include_discounted: bool,
        pay_only_due: bool,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentProcessRequest> {
        // Validate inputs
        if currency_code.len() != 3 {
            return Err(AtlasError::ValidationFailed("Currency code must be 3 characters".into()));
        }
        if !VALID_PAYMENT_METHODS.contains(&payment_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid payment method '{}'. Must be one of: {}", payment_method, VALID_PAYMENT_METHODS.join(", ")
            )));
        }
        if gl_date < payment_date {
            return Err(AtlasError::ValidationFailed("GL date cannot be before payment date".into()));
        }
        if let (Some(from), Some(to)) = (from_due_date, to_due_date) {
            if from > to {
                return Err(AtlasError::ValidationFailed("From due date cannot be after to due date".into()));
            }
        }
        if let Some(min) = min_amount {
            let v: f64 = min.parse().map_err(|_| AtlasError::ValidationFailed("Invalid min amount".into()))?;
            if v < 0.0 {
                return Err(AtlasError::ValidationFailed("Min amount cannot be negative".into()));
            }
        }
        if let Some(max) = max_amount {
            let v: f64 = max.parse().map_err(|_| AtlasError::ValidationFailed("Invalid max amount".into()))?;
            if v < 0.0 {
                return Err(AtlasError::ValidationFailed("Max amount cannot be negative".into()));
            }
        }
        if let (Some(min), Some(max)) = (min_amount, max_amount) {
            let min_v: f64 = min.parse().unwrap_or(0.0);
            let max_v: f64 = max.parse().unwrap_or(0.0);
            if min_v > max_v {
                return Err(AtlasError::ValidationFailed("Min amount cannot exceed max amount".into()));
            }
        }

        let ppr_number = format!("PPR-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        info!("Creating Payment Process Request {} for org {}", ppr_number, org_id);

        self.repository.create_ppr(
            org_id, &ppr_number, description,
            payment_date, gl_date, currency_code, payment_method,
            bank_account_id, bank_account_name, payment_format_code,
            supplier_id, supplier_name, from_due_date, to_due_date,
            min_amount, max_amount, include_discounted, pay_only_due,
            created_by,
        ).await
    }

    /// Get a PPR by ID.
    pub async fn get(&self, id: Uuid) -> AtlasResult<Option<PaymentProcessRequest>> {
        self.repository.get_ppr(id).await
    }

    /// Get a PPR by number.
    pub async fn get_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<PaymentProcessRequest>> {
        self.repository.get_ppr_by_number(org_id, number).await
    }

    /// List PPRs with optional status filter.
    pub async fn list(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PaymentProcessRequest>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_pprs(org_id, status).await
    }

    // ── Invoice selection ──

    /// Add an invoice to the PPR for payment.
    pub async fn add_invoice(
        &self,
        org_id: Uuid,
        ppr_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        supplier_id: Option<Uuid>,
        supplier_name: Option<&str>,
        invoice_amount: &str,
        amount_due: &str,
        discount_available: &str,
        payment_amount: &str,
        due_date: Option<NaiveDate>,
        invoice_payment_method: Option<&str>,
    ) -> AtlasResult<PprInvoiceLine> {
        let ppr = self.repository.get_ppr(ppr_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("PPR {} not found", ppr_id)))?;

        if ppr.status != "draft" && ppr.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot add invoices to PPR in '{}' status", ppr.status
            )));
        }

        let pay_amt: f64 = payment_amount.parse().map_err(|_| {
            AtlasError::ValidationFailed("Invalid payment amount".into())
        })?;
        if pay_amt <= 0.0 {
            return Err(AtlasError::ValidationFailed("Payment amount must be positive".into()));
        }

        let due: f64 = amount_due.parse().unwrap_or(0.0);
        if pay_amt > due + 0.01 {
            return Err(AtlasError::ValidationFailed(
                "Payment amount cannot exceed amount due".into()
            ));
        }

        info!("Adding invoice {} to PPR {}", invoice_number.unwrap_or("?"), ppr.ppr_number);

        let line = self.repository.add_invoice_line(
            org_id, ppr_id, invoice_id, invoice_number,
            supplier_id, supplier_name, invoice_amount,
            amount_due, discount_available, payment_amount,
            due_date, invoice_payment_method,
        ).await?;

        // Recalculate aggregates
        self.recalculate_aggregates(ppr_id).await?;

        Ok(line)
    }

    /// Exclude an invoice from the PPR.
    pub async fn exclude_invoice(&self, line_id: Uuid, reason: Option<&str>) -> AtlasResult<PprInvoiceLine> {
        let line = self.repository.get_invoice_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice line {} not found", line_id)))?;

        if line.status != "selected" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot exclude invoice in '{}' status", line.status
            )));
        }

        let ppr = self.repository.get_ppr(line.ppr_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("PPR not found".into()))?;

        if ppr.status != "draft" && ppr.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot modify invoices in PPR with '{}' status", ppr.status
            )));
        }

        info!("Excluding invoice from PPR: {}", reason.unwrap_or("No reason"));
        let updated = self.repository.update_invoice_line_status(line_id, "excluded", reason).await?;
        self.recalculate_aggregates(line.ppr_id).await?;
        Ok(updated)
    }

    /// Re-include a previously excluded invoice.
    pub async fn include_invoice(&self, line_id: Uuid) -> AtlasResult<PprInvoiceLine> {
        let line = self.repository.get_invoice_line(line_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice line {} not found", line_id)))?;

        if line.status != "excluded" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot include invoice in '{}' status. Must be 'excluded'.", line.status
            )));
        }

        info!("Re-including invoice in PPR");
        let updated = self.repository.update_invoice_line_status(line_id, "selected", None).await?;
        self.recalculate_aggregates(line.ppr_id).await?;
        Ok(updated)
    }

    /// List all invoice lines for a PPR.
    pub async fn list_invoices(&self, ppr_id: Uuid) -> AtlasResult<Vec<PprInvoiceLine>> {
        self.repository.list_invoice_lines(ppr_id).await
    }

    // ── Workflow ──

    /// Submit a draft PPR for processing.
    pub async fn submit(&self, id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> {
        let ppr = self.repository.get_ppr(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("PPR {} not found", id)))?;

        if ppr.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot submit PPR in '{}' status. Must be 'draft'.", ppr.status
            )));
        }

        // Must have at least one selected invoice
        let lines = self.repository.list_invoice_lines(id).await?;
        let selected = lines.iter().filter(|l| l.status == "selected").count();
        if selected == 0 {
            return Err(AtlasError::ValidationFailed(
                "PPR must have at least one selected invoice before submission".into()
            ));
        }

        info!("Submitting PPR {} ({} invoices selected)", ppr.ppr_number, selected);
        let _ = self.repository.update_ppr_submission(id, submitted_by).await?;
        self.repository.update_ppr_status(id, "submitted").await
    }

    /// Build payments from the selected invoices.
    /// Groups invoices by supplier and creates one payment per supplier.
    pub async fn build_payments(&self, id: Uuid, built_by: Option<Uuid>) -> AtlasResult<Vec<PprPayment>> {
        let ppr = self.repository.get_ppr(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("PPR {} not found", id)))?;

        if ppr.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot build payments for PPR in '{}' status. Must be 'submitted'.", ppr.status
            )));
        }

        // Remove any previous payments (rebuild)
        self.repository.remove_payments_by_ppr(id).await?;

        // Group selected invoices by supplier
        let lines = self.repository.list_invoice_lines(id).await?;
        let selected: Vec<&PprInvoiceLine> = lines.iter().filter(|l| l.status == "selected").collect();

        if selected.is_empty() {
            return Err(AtlasError::ValidationFailed("No selected invoices to build payments from".into()));
        }

        let mut supplier_groups: std::collections::HashMap<Uuid, Vec<&PprInvoiceLine>> = std::collections::HashMap::new();
        for line in &selected {
            let sid = line.supplier_id.unwrap_or_else(Uuid::new_v4);
            supplier_groups.entry(sid).or_default().push(line);
        }

        let mut payments = Vec::new();
        for (supplier_id, invoices) in supplier_groups {
            let total: f64 = invoices.iter().map(|i| i.payment_amount.parse::<f64>().unwrap_or(0.0)).sum();
            let supplier_name = invoices.first()
                .and_then(|i| i.supplier_name.clone())
                .unwrap_or_else(|| "Unknown".into());

            let payment_number = format!("PAY-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

            let payment = self.repository.create_payment(
                ppr.organization_id, id, supplier_id,
                &supplier_name, None, &payment_number,
                &ppr.payment_method, ppr.bank_account_id,
                &format!("{:.2}", total), invoices.len() as i32,
            ).await?;

            // Link invoice lines to payment
            for inv in invoices {
                let _ = self.repository.update_invoice_line_payment(inv.id, payment.id).await;
            }

            payments.push(payment);
        }

        // Update PPR aggregates and status
        let total_amount: f64 = payments.iter().map(|p| p.payment_amount.parse::<f64>().unwrap_or(0.0)).sum();
        self.repository.update_ppr_aggregates(
            id,
            selected.len() as i32,
            &format!("{:.2}", total_amount),
            payments.len() as i32,
            &format!("{:.2}", total_amount),
        ).await?;

        let _ = self.repository.update_ppr_built(id, built_by).await?;
        let _ = self.repository.update_ppr_status(id, "built").await?;

        info!("Built {} payments for PPR {} (total: {:.2})", payments.len(), ppr.ppr_number, total_amount);
        Ok(payments)
    }

    /// Format payments (generate payment documents/files).
    pub async fn format_payments(&self, id: Uuid) -> AtlasResult<PaymentProcessRequest> {
        let ppr = self.repository.get_ppr(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("PPR {} not found", id)))?;

        if ppr.status != "built" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot format payments for PPR in '{}' status. Must be 'built'.", ppr.status
            )));
        }

        // Mark all payments as formatted
        let payments = self.repository.list_payments(id).await?;
        for p in &payments {
            let doc_num = format!("DOC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
            let _ = self.repository.update_payment_formatted(p.id, Some(&doc_num)).await;
            let _ = self.repository.update_payment_status(p.id, "formatted").await;
        }

        info!("Formatted {} payments for PPR {}", payments.len(), ppr.ppr_number);
        self.repository.update_ppr_status(id, "formatted").await
    }

    /// Confirm formatted payments — marks them as confirmed.
    pub async fn confirm(&self, id: Uuid, confirmed_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> {
        let ppr = self.repository.get_ppr(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("PPR {} not found", id)))?;

        if ppr.status != "formatted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot confirm PPR in '{}' status. Must be 'formatted'.", ppr.status
            )));
        }

        // Mark all payments as confirmed, mark invoices as paid
        let payments = self.repository.list_payments(id).await?;
        for p in &payments {
            let _ = self.repository.update_payment_status(p.id, "confirmed").await;
        }

        let lines = self.repository.list_invoice_lines(id).await?;
        for line in lines.iter().filter(|l| l.status == "selected") {
            let _ = self.repository.update_invoice_line_status(line.id, "paid", None).await;
        }

        let _ = self.repository.update_ppr_confirmed(id, confirmed_by).await?;
        info!("Confirmed {} payments for PPR {}", payments.len(), ppr.ppr_number);
        self.repository.update_ppr_status(id, "confirmed").await
    }

    /// Complete the PPR — final step after all payments are reconciled.
    pub async fn complete(&self, id: Uuid, completed_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> {
        let ppr = self.repository.get_ppr(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("PPR {} not found", id)))?;

        if ppr.status != "confirmed" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete PPR in '{}' status. Must be 'confirmed'.", ppr.status
            )));
        }

        let _ = self.repository.update_ppr_completed(id, completed_by).await?;
        info!("Completing PPR {}", ppr.ppr_number);
        self.repository.update_ppr_status(id, "completed").await
    }

    /// Cancel a PPR (only from draft, submitted, or built status).
    pub async fn cancel(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<PaymentProcessRequest> {
        let ppr = self.repository.get_ppr(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("PPR {} not found", id)))?;

        if ppr.status != "draft" && ppr.status != "submitted" && ppr.status != "built" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot cancel PPR in '{}' status", ppr.status
            )));
        }

        info!("Cancelling PPR {}: {}", ppr.ppr_number, reason.unwrap_or("No reason"));
        let _ = self.repository.update_ppr_cancel_reason(id, reason).await?;
        self.repository.update_ppr_status(id, "cancelled").await
    }

    /// Reject a submitted PPR (e.g. by an approver).
    pub async fn reject(&self, id: Uuid, reason: &str) -> AtlasResult<PaymentProcessRequest> {
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed("Reject reason is required".into()));
        }

        let ppr = self.repository.get_ppr(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("PPR {} not found", id)))?;

        if ppr.status != "submitted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reject PPR in '{}' status. Must be 'submitted'.", ppr.status
            )));
        }

        info!("Rejecting PPR {}: {}", ppr.ppr_number, reason);
        let _ = self.repository.update_ppr_reject_reason(id, Some(reason)).await?;
        self.repository.update_ppr_status(id, "rejected").await
    }

    /// List payments for a PPR.
    pub async fn list_payments(&self, ppr_id: Uuid) -> AtlasResult<Vec<PprPayment>> {
        self.repository.list_payments(ppr_id).await
    }

    /// Get dashboard.
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PprDashboard> {
        self.repository.get_dashboard(org_id).await
    }

    // ── Helpers ──

    /// Recalculate the aggregate amounts from selected invoice lines.
    async fn recalculate_aggregates(&self, ppr_id: Uuid) -> AtlasResult<()> {
        let lines = self.repository.list_invoice_lines(ppr_id).await?;
        let selected: Vec<&PprInvoiceLine> = lines.iter().filter(|l| l.status == "selected").collect();
        let total_amount: f64 = selected.iter().map(|l| l.payment_amount.parse::<f64>().unwrap_or(0.0)).sum();

        // Payment count may be zero until build_payments runs
        let ppr = self.repository.get_ppr(ppr_id).await?;
        let payment_count = ppr.map(|p| p.total_payment_count).unwrap_or(0);

        self.repository.update_ppr_aggregates(
            ppr_id,
            selected.len() as i32,
            &format!("{:.2}", total_amount),
            payment_count,
            &format!("{:.2}", total_amount),
        ).await?;

        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// In-memory mock repository for testing.
    struct MockRepo {
        pprs: std::sync::Mutex<Vec<PaymentProcessRequest>>,
        invoice_lines: std::sync::Mutex<Vec<PprInvoiceLine>>,
        payments: std::sync::Mutex<Vec<PprPayment>>,
    }

    impl MockRepo {
        fn new() -> Self {
            Self {
                pprs: std::sync::Mutex::new(vec![]),
                invoice_lines: std::sync::Mutex::new(vec![]),
                payments: std::sync::Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl PprRepository for MockRepo {
        async fn create_ppr(&self, org_id: Uuid, ppr_number: &str, description: Option<&str>, payment_date: NaiveDate, gl_date: NaiveDate, currency_code: &str, payment_method: &str, bank_account_id: Option<Uuid>, bank_account_name: Option<&str>, payment_format_code: Option<&str>, supplier_id: Option<Uuid>, supplier_name: Option<&str>, from_due_date: Option<NaiveDate>, to_due_date: Option<NaiveDate>, min_amount: Option<&str>, max_amount: Option<&str>, include_discounted: bool, pay_only_due: bool, created_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> {
            let ppr = PaymentProcessRequest {
                id: Uuid::new_v4(), organization_id: org_id, ppr_number: ppr_number.into(),
                description: description.map(Into::into), payment_date, gl_date,
                currency_code: currency_code.into(), payment_method: payment_method.into(),
                bank_account_id, bank_account_name: bank_account_name.map(Into::into),
                payment_format_code: payment_format_code.map(Into::into),
                supplier_id, supplier_name: supplier_name.map(Into::into),
                from_due_date, to_due_date,
                min_amount: min_amount.map(Into::into), max_amount: max_amount.map(Into::into),
                include_discounted, pay_only_due,
                total_invoice_count: 0, total_invoice_amount: "0".into(),
                total_payment_count: 0, total_payment_amount: "0".into(),
                status: "draft".into(),
                submitted_by: None, submitted_at: None, built_by: None, built_at: None,
                confirmed_by: None, confirmed_at: None, completed_by: None, completed_at: None,
                cancelled_reason: None, reject_reason: None,
                metadata: serde_json::json!({}), created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.pprs.lock().unwrap().push(ppr.clone());
            Ok(ppr)
        }
        async fn get_ppr(&self, id: Uuid) -> AtlasResult<Option<PaymentProcessRequest>> {
            Ok(self.pprs.lock().unwrap().iter().find(|p| p.id == id).cloned())
        }
        async fn get_ppr_by_number(&self, org_id: Uuid, number: &str) -> AtlasResult<Option<PaymentProcessRequest>> {
            Ok(self.pprs.lock().unwrap().iter().find(|p| p.organization_id == org_id && p.ppr_number == number).cloned())
        }
        async fn list_pprs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PaymentProcessRequest>> {
            Ok(self.pprs.lock().unwrap().iter()
                .filter(|p| p.organization_id == org_id && (status.is_none() || p.status == status.unwrap()))
                .cloned().collect())
        }
        async fn update_ppr_status(&self, id: Uuid, status: &str) -> AtlasResult<PaymentProcessRequest> {
            let mut all = self.pprs.lock().unwrap();
            let ppr = all.iter_mut().find(|p| p.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ppr.status = status.into(); ppr.updated_at = Utc::now();
            Ok(ppr.clone())
        }
        async fn update_ppr_aggregates(&self, id: Uuid, inv_count: i32, inv_amt: &str, pay_count: i32, pay_amt: &str) -> AtlasResult<()> {
            let mut all = self.pprs.lock().unwrap();
            if let Some(ppr) = all.iter_mut().find(|p| p.id == id) {
                ppr.total_invoice_count = inv_count; ppr.total_invoice_amount = inv_amt.into();
                ppr.total_payment_count = pay_count; ppr.total_payment_amount = pay_amt.into();
                ppr.updated_at = Utc::now();
            }
            Ok(())
        }
        async fn update_ppr_submission(&self, id: Uuid, submitted_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> {
            let mut all = self.pprs.lock().unwrap();
            let ppr = all.iter_mut().find(|p| p.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ppr.submitted_by = submitted_by; ppr.submitted_at = Some(Utc::now()); ppr.updated_at = Utc::now();
            Ok(ppr.clone())
        }
        async fn update_ppr_built(&self, id: Uuid, built_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> {
            let mut all = self.pprs.lock().unwrap();
            let ppr = all.iter_mut().find(|p| p.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ppr.built_by = built_by; ppr.built_at = Some(Utc::now()); ppr.updated_at = Utc::now();
            Ok(ppr.clone())
        }
        async fn update_ppr_confirmed(&self, id: Uuid, confirmed_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> {
            let mut all = self.pprs.lock().unwrap();
            let ppr = all.iter_mut().find(|p| p.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ppr.confirmed_by = confirmed_by; ppr.confirmed_at = Some(Utc::now()); ppr.updated_at = Utc::now();
            Ok(ppr.clone())
        }
        async fn update_ppr_completed(&self, id: Uuid, completed_by: Option<Uuid>) -> AtlasResult<PaymentProcessRequest> {
            let mut all = self.pprs.lock().unwrap();
            let ppr = all.iter_mut().find(|p| p.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ppr.completed_by = completed_by; ppr.completed_at = Some(Utc::now()); ppr.updated_at = Utc::now();
            Ok(ppr.clone())
        }
        async fn update_ppr_cancel_reason(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<PaymentProcessRequest> {
            let mut all = self.pprs.lock().unwrap();
            let ppr = all.iter_mut().find(|p| p.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ppr.cancelled_reason = reason.map(Into::into); ppr.updated_at = Utc::now(); Ok(ppr.clone())
        }
        async fn update_ppr_reject_reason(&self, id: Uuid, reason: Option<&str>) -> AtlasResult<PaymentProcessRequest> {
            let mut all = self.pprs.lock().unwrap();
            let ppr = all.iter_mut().find(|p| p.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ppr.reject_reason = reason.map(Into::into); ppr.updated_at = Utc::now(); Ok(ppr.clone())
        }

        // Invoice lines
        async fn add_invoice_line(&self, org_id: Uuid, ppr_id: Uuid, invoice_id: Uuid, invoice_number: Option<&str>, supplier_id: Option<Uuid>, supplier_name: Option<&str>, invoice_amount: &str, amount_due: &str, discount_available: &str, payment_amount: &str, due_date: Option<NaiveDate>, payment_method: Option<&str>) -> AtlasResult<PprInvoiceLine> {
            let line = PprInvoiceLine {
                id: Uuid::new_v4(), organization_id: org_id, ppr_id, invoice_id,
                invoice_number: invoice_number.map(Into::into), supplier_id, supplier_name: supplier_name.map(Into::into),
                invoice_amount: invoice_amount.into(), amount_due: amount_due.into(),
                discount_available: discount_available.into(), payment_amount: payment_amount.into(),
                due_date, payment_method: payment_method.map(Into::into),
                status: "selected".into(), exclude_reason: None, payment_id: None,
                metadata: serde_json::json!({}), created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.invoice_lines.lock().unwrap().push(line.clone());
            Ok(line)
        }
        async fn get_invoice_line(&self, id: Uuid) -> AtlasResult<Option<PprInvoiceLine>> {
            Ok(self.invoice_lines.lock().unwrap().iter().find(|l| l.id == id).cloned())
        }
        async fn list_invoice_lines(&self, ppr_id: Uuid) -> AtlasResult<Vec<PprInvoiceLine>> {
            Ok(self.invoice_lines.lock().unwrap().iter().filter(|l| l.ppr_id == ppr_id).cloned().collect())
        }
        async fn update_invoice_line_status(&self, id: Uuid, status: &str, reason: Option<&str>) -> AtlasResult<PprInvoiceLine> {
            let mut all = self.invoice_lines.lock().unwrap();
            let line = all.iter_mut().find(|l| l.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            line.status = status.into(); line.exclude_reason = reason.map(Into::into); line.updated_at = Utc::now();
            Ok(line.clone())
        }
        async fn update_invoice_line_payment(&self, id: Uuid, payment_id: Uuid) -> AtlasResult<PprInvoiceLine> {
            let mut all = self.invoice_lines.lock().unwrap();
            let line = all.iter_mut().find(|l| l.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            line.payment_id = Some(payment_id); line.updated_at = Utc::now(); Ok(line.clone())
        }
        async fn remove_invoice_lines_by_ppr(&self, ppr_id: Uuid) -> AtlasResult<()> {
            self.invoice_lines.lock().unwrap().retain(|l| l.ppr_id != ppr_id); Ok(())
        }

        // Payments
        async fn create_payment(&self, org_id: Uuid, ppr_id: Uuid, supplier_id: Uuid, supplier_name: &str, supplier_site_id: Option<Uuid>, payment_number: &str, payment_method: &str, bank_account_id: Option<Uuid>, payment_amount: &str, invoice_count: i32) -> AtlasResult<PprPayment> {
            let payment = PprPayment {
                id: Uuid::new_v4(), organization_id: org_id, ppr_id, supplier_id,
                supplier_name: supplier_name.into(), supplier_site_id,
                payment_number: payment_number.into(), payment_method: payment_method.into(),
                bank_account_id, payment_amount: payment_amount.into(), invoice_count,
                status: "pending".into(), formatted_at: None, confirmed_at: None,
                document_number: None, metadata: serde_json::json!({}),
                created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.payments.lock().unwrap().push(payment.clone());
            Ok(payment)
        }
        async fn get_payment(&self, id: Uuid) -> AtlasResult<Option<PprPayment>> {
            Ok(self.payments.lock().unwrap().iter().find(|p| p.id == id).cloned())
        }
        async fn list_payments(&self, ppr_id: Uuid) -> AtlasResult<Vec<PprPayment>> {
            Ok(self.payments.lock().unwrap().iter().filter(|p| p.ppr_id == ppr_id).cloned().collect())
        }
        async fn update_payment_status(&self, id: Uuid, status: &str) -> AtlasResult<PprPayment> {
            let mut all = self.payments.lock().unwrap();
            let p = all.iter_mut().find(|p| p.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            p.status = status.into(); p.updated_at = Utc::now(); Ok(p.clone())
        }
        async fn update_payment_formatted(&self, id: Uuid, doc_number: Option<&str>) -> AtlasResult<PprPayment> {
            let mut all = self.payments.lock().unwrap();
            let p = all.iter_mut().find(|p| p.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            p.document_number = doc_number.map(Into::into); p.formatted_at = Some(Utc::now()); p.updated_at = Utc::now();
            Ok(p.clone())
        }
        async fn remove_payments_by_ppr(&self, ppr_id: Uuid) -> AtlasResult<()> {
            self.payments.lock().unwrap().retain(|p| p.ppr_id != ppr_id); Ok(())
        }

        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<PprDashboard> {
            let pprs = self.pprs.lock().unwrap();
            let payments = self.payments.lock().unwrap();
            let org_pprs: Vec<_> = pprs.iter().filter(|p| p.organization_id == org_id).collect();
            let org_payments: Vec<_> = payments.iter().filter(|p| p.organization_id == org_id && p.status == "confirmed").collect();
            let total_paid: f64 = org_payments.iter().map(|p| p.payment_amount.parse::<f64>().unwrap_or(0.0)).sum();
            Ok(PprDashboard {
                organization_id: org_id,
                total_pprs: org_pprs.len() as i32,
                draft_pprs: org_pprs.iter().filter(|p| p.status == "draft").count() as i32,
                pending_pprs: org_pprs.iter().filter(|p| p.status == "submitted" || p.status == "built" || p.status == "formatted").count() as i32,
                completed_pprs: org_pprs.iter().filter(|p| p.status == "completed").count() as i32,
                total_payments_made: org_payments.len() as i32,
                total_amount_paid: format!("{:.2}", total_paid),
                by_payment_method: serde_json::json!([]),
            })
        }
    }

    fn eng() -> PaymentProcessRequestEngine {
        PaymentProcessRequestEngine::new(Arc::new(MockRepo::new()))
    }

    fn today() -> NaiveDate { Utc::now().date_naive() }
    fn tomorrow() -> NaiveDate { today() + chrono::Duration::days(1) }

    // ── Create tests ──

    #[tokio::test]
    async fn test_create_valid() {
        let ppr = eng().create(
            Uuid::new_v4(), Some("Weekly pay run"), today(), tomorrow(),
            "USD", "electronic", Some(Uuid::new_v4()), Some("Main Operating"), Some("EFT_US"),
            None, None, None, Some(today()), None, None,
            true, false, None,
        ).await.unwrap();
        assert_eq!(ppr.status, "draft");
        assert_eq!(ppr.payment_method, "electronic");
        assert_eq!(ppr.currency_code, "USD");
        assert_eq!(ppr.total_invoice_count, 0);
    }

    #[tokio::test]
    async fn test_create_invalid_currency() {
        assert!(eng().create(
            Uuid::new_v4(), None, today(), tomorrow(), "US", "electronic",
            None, None, None, None, None, None, None, None, None, true, false, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_invalid_payment_method() {
        assert!(eng().create(
            Uuid::new_v4(), None, today(), tomorrow(), "USD", "crypto",
            None, None, None, None, None, None, None, None, None, true, false, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_gl_date_before_payment_date() {
        assert!(eng().create(
            Uuid::new_v4(), None, tomorrow(), today(), "USD", "check",
            None, None, None, None, None, None, None, None, None, true, false, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_from_after_to_due_date() {
        assert!(eng().create(
            Uuid::new_v4(), None, today(), tomorrow(), "USD", "check",
            None, None, None, None, None,
            Some(today()), Some(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            None, None, true, false, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_min_exceeds_max() {
        assert!(eng().create(
            Uuid::new_v4(), None, today(), tomorrow(), "USD", "check",
            None, None, None, None, None, None, None,
            Some("10000"), Some("1000"), true, false, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_create_negative_min_amount() {
        assert!(eng().create(
            Uuid::new_v4(), None, today(), tomorrow(), "USD", "check",
            None, None, None, None, None, None, None,
            Some("-100"), None, true, false, None,
        ).await.is_err());
    }

    // ── Invoice selection tests ──

    #[tokio::test]
    async fn test_add_invoice() {
        let e = eng();
        let org = Uuid::new_v4();
        let supplier = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();

        let line = e.add_invoice(
            org, ppr.id, Uuid::new_v4(), Some("INV-001"),
            Some(supplier), Some("Acme Corp"), "5000.00", "5000.00",
            "0", "5000.00", Some(today()), Some("check"),
        ).await.unwrap();

        assert_eq!(line.status, "selected");
        assert_eq!(line.payment_amount, "5000.00");

        let updated_ppr = e.get(ppr.id).await.unwrap().unwrap();
        assert_eq!(updated_ppr.total_invoice_count, 1);
    }

    #[tokio::test]
    async fn test_add_invoice_zero_amount() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();

        assert!(e.add_invoice(
            org, ppr.id, Uuid::new_v4(), None, None, None, "1000", "1000", "0", "0", None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_add_invoice_exceeds_due() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();

        assert!(e.add_invoice(
            org, ppr.id, Uuid::new_v4(), None, None, None, "1000", "1000", "0", "2000", None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_add_invoice_to_non_draft() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        // Add an invoice so we can submit
        let _ = e.add_invoice(org, ppr.id, Uuid::new_v4(), Some("INV-1"), None, Some("S"), "100", "100", "0", "100", None, None).await.unwrap();
        let _ = e.submit(ppr.id, None).await.unwrap();
        let _ = e.build_payments(ppr.id, None).await.unwrap();

        assert!(e.add_invoice(
            org, ppr.id, Uuid::new_v4(), None, None, None, "100", "100", "0", "100", None, None,
        ).await.is_err());
    }

    #[tokio::test]
    async fn test_exclude_include_invoice() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        let line = e.add_invoice(org, ppr.id, Uuid::new_v4(), Some("INV-1"), None, None, "100", "100", "0", "100", None, None).await.unwrap();

        let excluded = e.exclude_invoice(line.id, Some("Duplicate")).await.unwrap();
        assert_eq!(excluded.status, "excluded");

        let ppr_after = e.get(ppr.id).await.unwrap().unwrap();
        assert_eq!(ppr_after.total_invoice_count, 0);

        let included = e.include_invoice(line.id).await.unwrap();
        assert_eq!(included.status, "selected");

        let ppr_after2 = e.get(ppr.id).await.unwrap().unwrap();
        assert_eq!(ppr_after2.total_invoice_count, 1);
    }

    #[tokio::test]
    async fn test_exclude_not_selected() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        let line = e.add_invoice(org, ppr.id, Uuid::new_v4(), None, None, None, "100", "100", "0", "100", None, None).await.unwrap();
        let _ = e.exclude_invoice(line.id, Some("reason")).await.unwrap();
        assert!(e.exclude_invoice(line.id, Some("again")).await.is_err());
    }

    #[tokio::test]
    async fn test_include_not_excluded() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        let line = e.add_invoice(org, ppr.id, Uuid::new_v4(), None, None, None, "100", "100", "0", "100", None, None).await.unwrap();
        assert!(e.include_invoice(line.id).await.is_err());
    }

    // ── Workflow tests ──

    #[tokio::test]
    async fn test_submit_no_invoices() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        assert!(e.submit(ppr.id, None).await.is_err());
    }

    #[tokio::test]
    async fn test_submit_not_draft() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        let _ = e.add_invoice(org, ppr.id, Uuid::new_v4(), None, None, None, "100", "100", "0", "100", None, None).await.unwrap();
        let _ = e.submit(ppr.id, None).await.unwrap();
        assert!(e.submit(ppr.id, None).await.is_err());
    }

    #[tokio::test]
    async fn test_full_lifecycle_single_supplier() {
        let e = eng();
        let org = Uuid::new_v4();
        let supplier = Uuid::new_v4();

        // Create PPR
        let ppr = e.create(org, Some("Weekly pay run"), today(), tomorrow(), "USD", "electronic", Some(Uuid::new_v4()), Some("Op Acct"), Some("EFT_US"), Some(supplier), Some("Acme Corp"), None, None, None, None, true, false, None).await.unwrap();
        assert_eq!(ppr.status, "draft");

        // Add invoices
        let inv1 = e.add_invoice(org, ppr.id, Uuid::new_v4(), Some("INV-001"), Some(supplier), Some("Acme Corp"), "5000", "5000", "100", "5000", Some(today()), Some("electronic")).await.unwrap();
        let inv2 = e.add_invoice(org, ppr.id, Uuid::new_v4(), Some("INV-002"), Some(supplier), Some("Acme Corp"), "3000", "3000", "0", "3000", Some(today()), Some("electronic")).await.unwrap();

        let ppr_check = e.get(ppr.id).await.unwrap().unwrap();
        assert_eq!(ppr_check.total_invoice_count, 2);

        // Submit
        let submitted = e.submit(ppr.id, Some(Uuid::new_v4())).await.unwrap();
        assert_eq!(submitted.status, "submitted");

        // Build payments
        let payments = e.build_payments(ppr.id, Some(Uuid::new_v4())).await.unwrap();
        assert_eq!(payments.len(), 1); // One supplier => one payment
        assert_eq!(payments[0].invoice_count, 2);
        assert_eq!(payments[0].payment_amount, "8000.00");

        let built = e.get(ppr.id).await.unwrap().unwrap();
        assert_eq!(built.status, "built");
        assert_eq!(built.total_payment_count, 1);
        assert_eq!(built.total_payment_amount, "8000.00");

        // Format
        let formatted = e.format_payments(ppr.id).await.unwrap();
        assert_eq!(formatted.status, "formatted");

        let pay = e.list_payments(ppr.id).await.unwrap();
        assert_eq!(pay[0].status, "formatted");
        assert!(pay[0].document_number.is_some());

        // Confirm
        let confirmed = e.confirm(ppr.id, Some(Uuid::new_v4())).await.unwrap();
        assert_eq!(confirmed.status, "confirmed");

        // Check invoices are marked paid
        let lines = e.list_invoices(ppr.id).await.unwrap();
        assert!(lines.iter().all(|l| l.status == "paid"));

        // Complete
        let completed = e.complete(ppr.id, Some(Uuid::new_v4())).await.unwrap();
        assert_eq!(completed.status, "completed");

        // Dashboard
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_pprs, 1);
        assert_eq!(dash.completed_pprs, 1);
        assert_eq!(dash.total_payments_made, 1);
    }

    #[tokio::test]
    async fn test_full_lifecycle_multiple_suppliers() {
        let e = eng();
        let org = Uuid::new_v4();
        let s1 = Uuid::new_v4();
        let s2 = Uuid::new_v4();
        let s3 = Uuid::new_v4();

        let ppr = e.create(org, None, today(), tomorrow(), "USD", "ach", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();

        e.add_invoice(org, ppr.id, Uuid::new_v4(), Some("INV-A1"), Some(s1), Some("Alpha"), "2000", "2000", "0", "2000", None, None).await.unwrap();
        e.add_invoice(org, ppr.id, Uuid::new_v4(), Some("INV-A2"), Some(s1), Some("Alpha"), "1500", "1500", "0", "1500", None, None).await.unwrap();
        e.add_invoice(org, ppr.id, Uuid::new_v4(), Some("INV-B1"), Some(s2), Some("Beta"), "8000", "8000", "0", "8000", None, None).await.unwrap();
        e.add_invoice(org, ppr.id, Uuid::new_v4(), Some("INV-C1"), Some(s3), Some("Gamma"), "3000", "3000", "0", "3000", None, None).await.unwrap();
        // Exclude one
        let inv_c1 = e.list_invoices(ppr.id).await.unwrap().into_iter().find(|l| l.invoice_number.as_deref() == Some("INV-C1")).unwrap();
        e.exclude_invoice(inv_c1.id, Some("On hold")).await.unwrap();

        e.submit(ppr.id, None).await.unwrap();
        let payments = e.build_payments(ppr.id, None).await.unwrap();
        assert_eq!(payments.len(), 2); // s1 + s2 (s3 excluded)

        let total: f64 = payments.iter().map(|p| p.payment_amount.parse::<f64>().unwrap_or(0.0)).sum();
        assert_eq!(total, 11500.0); // 3500 + 8000

        e.format_payments(ppr.id).await.unwrap();
        e.confirm(ppr.id, None).await.unwrap();
        e.complete(ppr.id, None).await.unwrap();
    }

    #[tokio::test]
    async fn test_build_not_submitted() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        assert!(e.build_payments(ppr.id, None).await.is_err());
    }

    #[tokio::test]
    async fn test_format_not_built() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        assert!(e.format_payments(ppr.id).await.is_err());
    }

    #[tokio::test]
    async fn test_confirm_not_formatted() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        assert!(e.confirm(ppr.id, None).await.is_err());
    }

    #[tokio::test]
    async fn test_complete_not_confirmed() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        assert!(e.complete(ppr.id, None).await.is_err());
    }

    #[tokio::test]
    async fn test_cancel_draft() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        let cancelled = e.cancel(ppr.id, Some("No longer needed")).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_submitted() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        let _ = e.add_invoice(org, ppr.id, Uuid::new_v4(), None, None, None, "100", "100", "0", "100", None, None).await.unwrap();
        let _ = e.submit(ppr.id, None).await.unwrap();
        let cancelled = e.cancel(ppr.id, Some("Wrong batch")).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_confirmed_fails() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        let _ = e.add_invoice(org, ppr.id, Uuid::new_v4(), None, Some(Uuid::new_v4()), Some("S"), "100", "100", "0", "100", None, None).await.unwrap();
        let _ = e.submit(ppr.id, None).await.unwrap();
        let _ = e.build_payments(ppr.id, None).await.unwrap();
        let _ = e.format_payments(ppr.id).await.unwrap();
        let _ = e.confirm(ppr.id, None).await.unwrap();
        assert!(e.cancel(ppr.id, None).await.is_err());
    }

    #[tokio::test]
    async fn test_reject() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        let _ = e.add_invoice(org, ppr.id, Uuid::new_v4(), None, None, None, "100", "100", "0", "100", None, None).await.unwrap();
        let _ = e.submit(ppr.id, None).await.unwrap();
        let rejected = e.reject(ppr.id, "Insufficient funds").await.unwrap();
        assert_eq!(rejected.status, "rejected");
    }

    #[tokio::test]
    async fn test_reject_empty_reason() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        let _ = e.add_invoice(org, ppr.id, Uuid::new_v4(), None, None, None, "100", "100", "0", "100", None, None).await.unwrap();
        let _ = e.submit(ppr.id, None).await.unwrap();
        assert!(e.reject(ppr.id, "").await.is_err());
    }

    #[tokio::test]
    async fn test_reject_not_submitted() {
        let e = eng();
        let org = Uuid::new_v4();
        let ppr = e.create(org, None, today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        assert!(e.reject(ppr.id, "reason").await.is_err());
    }

    #[tokio::test]
    async fn test_list_invalid_status() {
        assert!(eng().list(Uuid::new_v4(), Some("invalid")).await.is_err());
    }

    #[tokio::test]
    async fn test_list_valid_status() {
        assert!(eng().list(Uuid::new_v4(), Some("draft")).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let s1 = Uuid::new_v4();

        // PPR 1 - complete
        let ppr1 = e.create(org, Some("Run 1"), today(), tomorrow(), "USD", "electronic", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();
        e.add_invoice(org, ppr1.id, Uuid::new_v4(), Some("INV-1"), Some(s1), Some("Acme"), "5000", "5000", "0", "5000", None, None).await.unwrap();
        e.submit(ppr1.id, None).await.unwrap();
        e.build_payments(ppr1.id, None).await.unwrap();
        e.format_payments(ppr1.id).await.unwrap();
        e.confirm(ppr1.id, None).await.unwrap();
        e.complete(ppr1.id, None).await.unwrap();

        // PPR 2 - draft
        let _ppr2 = e.create(org, Some("Run 2"), today(), tomorrow(), "USD", "check", None, None, None, None, None, None, None, None, None, true, false, None).await.unwrap();

        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_pprs, 2);
        assert_eq!(dash.draft_pprs, 1);
        assert_eq!(dash.completed_pprs, 1);
        assert_eq!(dash.total_payments_made, 1);
    }

    #[test]
    fn test_valid_status_constants() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"submitted"));
        assert!(VALID_STATUSES.contains(&"built"));
        assert!(VALID_STATUSES.contains(&"formatted"));
        assert!(VALID_STATUSES.contains(&"confirmed"));
        assert!(VALID_STATUSES.contains(&"completed"));
        assert!(VALID_STATUSES.contains(&"cancelled"));
        assert!(VALID_STATUSES.contains(&"rejected"));
    }

    #[test]
    fn test_valid_payment_method_constants() {
        assert!(VALID_PAYMENT_METHODS.contains(&"check"));
        assert!(VALID_PAYMENT_METHODS.contains(&"electronic"));
        assert!(VALID_PAYMENT_METHODS.contains(&"wire"));
        assert!(VALID_PAYMENT_METHODS.contains(&"ach"));
        assert!(VALID_PAYMENT_METHODS.contains(&"sepa"));
        assert!(VALID_PAYMENT_METHODS.contains(&"manual"));
    }
}
