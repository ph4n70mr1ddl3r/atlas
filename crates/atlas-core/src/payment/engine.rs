//! Payment Management Engine
//!
//! Manages payment terms, payment batches, payment processing,
//! scheduled payments, void/reissue, and remittance advice.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Payables > Payments

use atlas_shared::{
    PaymentTerm, PaymentBatch, Payment, PaymentLine, ScheduledPayment,
    PaymentFormat, RemittanceAdvice, PaymentDashboardSummary,
    AtlasError, AtlasResult,
};
use super::PaymentRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid payment methods
const VALID_PAYMENT_METHODS: &[&str] = &[
    "check", "eft", "wire", "ach",
];

/// Valid payment statuses
const VALID_PAYMENT_STATUSES: &[&str] = &[
    "draft", "issued", "cleared", "voided", "reconciled", "stopped",
];

/// Valid batch statuses
const VALID_BATCH_STATUSES: &[&str] = &[
    "draft", "selected", "approved", "formatted", "confirmed", "cancelled",
];

/// Valid scheduled payment statuses
const VALID_SCHEDULED_STATUSES: &[&str] = &[
    "pending", "selected", "paid", "cancelled",
];

/// Valid installment frequencies
const VALID_FREQUENCIES: &[&str] = &[
    "monthly", "quarterly", "weekly",
];

/// Valid remittance delivery methods
const VALID_DELIVERY_METHODS: &[&str] = &[
    "email", "print", "edi", "xml",
];

/// Valid format types
const VALID_FORMAT_TYPES: &[&str] = &[
    "file", "printed_check", "edi", "xml", "json",
];

/// Payment Management engine
pub struct PaymentEngine {
    repository: Arc<dyn PaymentRepository>,
}

impl PaymentEngine {
    pub fn new(repository: Arc<dyn PaymentRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Payment Terms
    // ========================================================================

    /// Create or update a payment term
    pub async fn create_payment_term(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        due_days: i32,
        discount_days: Option<i32>,
        discount_percentage: Option<&str>,
        is_installment: bool,
        installment_count: Option<i32>,
        installment_frequency: Option<&str>,
        default_payment_method: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentTerm> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Payment term code and name are required".to_string(),
            ));
        }
        if due_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "Due days must be non-negative".to_string(),
            ));
        }
        if let Some(method) = default_payment_method {
            if !VALID_PAYMENT_METHODS.contains(&method) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid payment method '{}'. Must be one of: {}",
                    method, VALID_PAYMENT_METHODS.join(", ")
                )));
            }
        }
        if is_installment {
            if let Some(freq) = installment_frequency {
                if !VALID_FREQUENCIES.contains(&freq) {
                    return Err(AtlasError::ValidationFailed(format!(
                        "Invalid frequency '{}'. Must be one of: {}",
                        freq, VALID_FREQUENCIES.join(", ")
                    )));
                }
            }
            if installment_count.unwrap_or(0) < 2 {
                return Err(AtlasError::ValidationFailed(
                    "Installment terms must have at least 2 installments".to_string(),
                ));
            }
        }

        info!("Creating/updating payment term {} in org {}", code, org_id);

        self.repository.create_payment_term(
            org_id, code, name, description, due_days,
            discount_days, discount_percentage,
            is_installment, installment_count, installment_frequency,
            default_payment_method, effective_from, effective_to,
            created_by,
        ).await
    }

    /// Get a payment term by code
    pub async fn get_payment_term(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<PaymentTerm>> {
        self.repository.get_payment_term(org_id, code).await
    }

    /// List all payment terms for an organization
    pub async fn list_payment_terms(&self, org_id: Uuid) -> AtlasResult<Vec<PaymentTerm>> {
        self.repository.list_payment_terms(org_id).await
    }

    /// Delete (soft-delete) a payment term
    pub async fn delete_payment_term(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.get_payment_term(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payment term '{}' not found", code)
            ))?;

        info!("Deleting payment term {} in org {}", code, org_id);
        self.repository.delete_payment_term(org_id, code).await
    }

    // ========================================================================
    // Payment Batches
    // ========================================================================

    /// Create a new payment batch
    pub async fn create_payment_batch(
        &self,
        org_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        payment_date: chrono::NaiveDate,
        bank_account_id: Option<Uuid>,
        payment_method: &str,
        currency_code: &str,
        selection_criteria: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<PaymentBatch> {
        if !VALID_PAYMENT_METHODS.contains(&payment_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid payment method '{}'. Must be one of: {}",
                payment_method, VALID_PAYMENT_METHODS.join(", ")
            )));
        }

        let batch_number = format!("PB-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating payment batch {} for org {}", batch_number, org_id);

        self.repository.create_payment_batch(
            org_id, &batch_number, name, description,
            payment_date, bank_account_id, payment_method,
            currency_code, selection_criteria, created_by,
        ).await
    }

    /// Get a payment batch
    pub async fn get_payment_batch(&self, id: Uuid) -> AtlasResult<Option<PaymentBatch>> {
        self.repository.get_payment_batch_by_id(id).await
    }

    /// List payment batches
    pub async fn list_payment_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<PaymentBatch>> {
        if let Some(s) = status {
            if !VALID_BATCH_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid batch status '{}'. Must be one of: {}",
                    s, VALID_BATCH_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_payment_batches(org_id, status).await
    }

    /// Transition a payment batch through its workflow
    /// Draft → Selected → Approved → Formatted → Confirmed (or Cancelled from any)
    pub async fn transition_batch(
        &self,
        batch_id: Uuid,
        new_status: &str,
        action_by: Option<Uuid>,
        cancellation_reason: Option<&str>,
    ) -> AtlasResult<PaymentBatch> {
        let batch = self.repository.get_payment_batch_by_id(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payment batch {} not found", batch_id)
            ))?;

        // Validate transition
        let valid = match new_status {
            "selected" => batch.status == "draft",
            "approved" => batch.status == "selected",
            "formatted" => batch.status == "approved",
            "confirmed" => batch.status == "formatted",
            "cancelled" => batch.status != "confirmed" && batch.status != "cancelled",
            _ => false,
        };

        if !valid {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot transition batch from '{}' to '{}'",
                batch.status, new_status
            )));
        }

        info!("Transitioning batch {} from {} to {}", batch.batch_number, batch.status, new_status);

        self.repository.update_payment_batch_status(
            batch_id, new_status, action_by, cancellation_reason,
        ).await
    }

    // ========================================================================
    // Payments
    // ========================================================================

    /// Create a payment (typically within a batch)
    pub async fn create_payment(
        &self,
        org_id: Uuid,
        batch_id: Option<Uuid>,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_site: Option<&str>,
        payment_date: chrono::NaiveDate,
        payment_method: &str,
        currency_code: &str,
        payment_amount: &str,
        discount_taken: &str,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        cash_account_code: Option<&str>,
        ap_account_code: Option<&str>,
        discount_account_code: Option<&str>,
        check_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<Payment> {
        if !VALID_PAYMENT_METHODS.contains(&payment_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid payment method '{}'. Must be one of: {}",
                payment_method, VALID_PAYMENT_METHODS.join(", ")
            )));
        }

        let amount: f64 = payment_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Payment amount must be a valid number".to_string(),
        ))?;
        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Payment amount must be positive".to_string(),
            ));
        }

        let payment_number = format!("PAY-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating payment {} for supplier {}", payment_number, supplier_id);

        self.repository.create_payment(
            org_id, &payment_number, batch_id,
            supplier_id, supplier_number, supplier_name, supplier_site,
            payment_date, payment_method, currency_code,
            payment_amount, discount_taken,
            bank_account_id, bank_account_name,
            cash_account_code, ap_account_code, discount_account_code,
            check_number, created_by,
        ).await
    }

    /// Get a payment by ID
    pub async fn get_payment(&self, id: Uuid) -> AtlasResult<Option<Payment>> {
        self.repository.get_payment(id).await
    }

    /// List payments with optional filters
    pub async fn list_payments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        supplier_id: Option<Uuid>,
        batch_id: Option<Uuid>,
    ) -> AtlasResult<Vec<Payment>> {
        if let Some(s) = status {
            if !VALID_PAYMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid payment status '{}'. Must be one of: {}",
                    s, VALID_PAYMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_payments(org_id, status, supplier_id, batch_id).await
    }

    /// Issue a payment (transition from draft to issued)
    pub async fn issue_payment(&self, payment_id: Uuid) -> AtlasResult<Payment> {
        let payment = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payment {} not found", payment_id)
            ))?;

        if payment.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot issue payment in '{}' status. Must be 'draft'.",
                payment.status
            )));
        }

        info!("Issuing payment {}", payment.payment_number);
        self.repository.update_payment_status(
            payment_id, "issued", None, None, None, None,
        ).await
    }

    /// Clear a payment (bank has cleared it)
    pub async fn clear_payment(&self, payment_id: Uuid, cleared_by: Option<Uuid>) -> AtlasResult<Payment> {
        let payment = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payment {} not found", payment_id)
            ))?;

        if payment.status != "issued" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot clear payment in '{}' status. Must be 'issued'.",
                payment.status
            )));
        }

        info!("Clearing payment {}", payment.payment_number);
        self.repository.update_payment_status(
            payment_id, "cleared",
            Some(chrono::Utc::now().date_naive()),
            cleared_by, None, None,
        ).await
    }

    /// Void a payment
    pub async fn void_payment(&self, payment_id: Uuid, voided_by: Uuid, reason: &str) -> AtlasResult<Payment> {
        let payment = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payment {} not found", payment_id)
            ))?;

        if payment.status == "voided" {
            return Err(AtlasError::WorkflowError(
                "Payment is already voided".to_string(),
            ));
        }
        if payment.status == "reconciled" {
            return Err(AtlasError::WorkflowError(
                "Cannot void a reconciled payment".to_string(),
            ));
        }
        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Void reason is required".to_string(),
            ));
        }

        info!("Voiding payment {} - reason: {}", payment.payment_number, reason);
        self.repository.update_payment_status(
            payment_id, "voided", None, None, Some(reason), Some(voided_by),
        ).await
    }

    // ========================================================================
    // Payment Lines
    // ========================================================================

    /// Add a payment line (link invoice to payment)
    pub async fn add_payment_line(
        &self,
        org_id: Uuid,
        payment_id: Uuid,
        line_number: i32,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        invoice_date: Option<chrono::NaiveDate>,
        invoice_due_date: Option<chrono::NaiveDate>,
        invoice_amount: Option<&str>,
        amount_paid: &str,
        discount_taken: &str,
        withholding_amount: &str,
    ) -> AtlasResult<PaymentLine> {
        let paid: f64 = amount_paid.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount paid must be a valid number".to_string(),
        ))?;
        if paid <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Amount paid must be positive".to_string(),
            ));
        }

        self.repository.create_payment_line(
            org_id, payment_id, line_number,
            invoice_id, invoice_number, invoice_date, invoice_due_date,
            invoice_amount, amount_paid, discount_taken, withholding_amount,
        ).await
    }

    /// List payment lines for a payment
    pub async fn list_payment_lines(&self, payment_id: Uuid) -> AtlasResult<Vec<PaymentLine>> {
        self.repository.list_payment_lines(payment_id).await
    }

    // ========================================================================
    // Scheduled Payments
    // ========================================================================

    /// Create a scheduled payment
    pub async fn create_scheduled_payment(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        invoice_number: Option<&str>,
        supplier_id: Uuid,
        supplier_name: Option<&str>,
        scheduled_payment_date: chrono::NaiveDate,
        scheduled_amount: &str,
        installment_number: i32,
        payment_method: Option<&str>,
        bank_account_id: Option<Uuid>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ScheduledPayment> {
        let amount: f64 = scheduled_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Scheduled amount must be a valid number".to_string(),
        ))?;
        if amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Scheduled amount must be positive".to_string(),
            ));
        }
        if let Some(method) = payment_method {
            if !VALID_PAYMENT_METHODS.contains(&method) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid payment method '{}'. Must be one of: {}",
                    method, VALID_PAYMENT_METHODS.join(", ")
                )));
            }
        }

        info!("Creating scheduled payment for invoice {} due {}",
            invoice_id, scheduled_payment_date);

        self.repository.create_scheduled_payment(
            org_id, invoice_id, invoice_number,
            supplier_id, supplier_name,
            scheduled_payment_date, scheduled_amount,
            installment_number, payment_method, bank_account_id,
            created_by,
        ).await
    }

    /// List scheduled payments
    pub async fn list_scheduled_payments(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        supplier_id: Option<Uuid>,
    ) -> AtlasResult<Vec<ScheduledPayment>> {
        if let Some(s) = status {
            if !VALID_SCHEDULED_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_SCHEDULED_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_scheduled_payments(org_id, status, supplier_id).await
    }

    /// Calculate early payment discount for a given payment term
    /// Returns the discount amount if paid within the discount period
    pub fn calculate_early_payment_discount(
        &self,
        term: &PaymentTerm,
        invoice_amount: &str,
        days_since_invoice: i32,
    ) -> Option<String> {
        let discount_days = term.discount_days?;
        let discount_pct_str = term.discount_percentage.as_ref()?;
        let discount_pct: f64 = discount_pct_str.parse().ok()?;
        let amount: f64 = invoice_amount.parse().ok()?;

        if days_since_invoice <= discount_days && discount_pct > 0.0 {
            Some(format!("{:.2}", amount * discount_pct / 100.0))
        } else {
            None
        }
    }

    /// Calculate the due date for an invoice given a payment term
    pub fn calculate_due_date(&self, term: &PaymentTerm, invoice_date: chrono::NaiveDate) -> chrono::NaiveDate {
        invoice_date + chrono::Duration::days(term.due_days as i64)
    }

    // ========================================================================
    // Payment Formats
    // ========================================================================

    /// Create a payment format
    pub async fn create_payment_format(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        format_type: &str,
        template_reference: Option<&str>,
        applicable_methods: serde_json::Value,
    ) -> AtlasResult<PaymentFormat> {
        if !VALID_FORMAT_TYPES.contains(&format_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid format type '{}'. Must be one of: {}",
                format_type, VALID_FORMAT_TYPES.join(", ")
            )));
        }

        self.repository.create_payment_format(
            org_id, code, name, description,
            format_type, template_reference,
            applicable_methods, false,
        ).await
    }

    /// List payment formats
    pub async fn list_payment_formats(&self, org_id: Uuid) -> AtlasResult<Vec<PaymentFormat>> {
        self.repository.list_payment_formats(org_id).await
    }

    // ========================================================================
    // Remittance Advice
    // ========================================================================

    /// Create a remittance advice for a payment
    pub async fn create_remittance_advice(
        &self,
        org_id: Uuid,
        payment_id: Uuid,
        delivery_method: &str,
        delivery_address: Option<&str>,
        contact_name: Option<&str>,
        contact_email: Option<&str>,
        subject: Option<&str>,
        body: Option<&str>,
        payment_summary: serde_json::Value,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RemittanceAdvice> {
        if !VALID_DELIVERY_METHODS.contains(&delivery_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid delivery method '{}'. Must be one of: {}",
                delivery_method, VALID_DELIVERY_METHODS.join(", ")
            )));
        }

        // Verify payment exists
        let payment = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payment {} not found", payment_id)
            ))?;

        if payment.status == "draft" {
            return Err(AtlasError::ValidationFailed(
                "Cannot create remittance advice for a draft payment".to_string(),
            ));
        }

        info!("Creating remittance advice for payment {}", payment.payment_number);

        self.repository.create_remittance_advice(
            org_id, payment_id, delivery_method, delivery_address,
            contact_name, contact_email, subject, body,
            payment_summary, created_by,
        ).await
    }

    /// List remittance advices
    pub async fn list_remittance_advices(
        &self,
        org_id: Uuid,
        payment_id: Option<Uuid>,
    ) -> AtlasResult<Vec<RemittanceAdvice>> {
        self.repository.list_remittance_advices(org_id, payment_id).await
    }

    // ========================================================================
    // Dashboard / Reporting
    // ========================================================================

    /// Generate a payment dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<PaymentDashboardSummary> {
        let all_payments = self.repository.list_payments(org_id, None, None, None).await?;
        let scheduled = self.repository.list_scheduled_payments(org_id, Some("pending"), None).await?;

        let mut total_pending_count = 0i32;
        let mut total_pending_amount = 0.0f64;
        let mut total_paid_count = 0i32;
        let mut total_paid_amount = 0.0f64;
        let mut total_discount = 0.0f64;

        for p in &all_payments {
            match p.status.as_str() {
                "draft" | "issued" => {
                    total_pending_count += 1;
                    total_pending_amount += p.payment_amount.parse::<f64>().unwrap_or(0.0);
                }
                "cleared" | "reconciled" => {
                    total_paid_count += 1;
                    total_paid_amount += p.payment_amount.parse::<f64>().unwrap_or(0.0);
                }
                _ => {}
            }
            total_discount += p.discount_taken.parse::<f64>().unwrap_or(0.0);
        }

        // Count upcoming scheduled payments (next 7 days)
        let today = chrono::Utc::now().date_naive();
        let seven_days = today + chrono::Duration::days(7);
        let mut upcoming_count = 0i32;
        let mut upcoming_amount = 0.0f64;

        for s in &scheduled {
            if s.scheduled_payment_date <= seven_days {
                upcoming_count += 1;
                upcoming_amount += s.scheduled_amount.parse::<f64>().unwrap_or(0.0);
            }
        }

        // Group by method
        let mut by_method: std::collections::HashMap<String, (i32, f64)> = std::collections::HashMap::new();
        for p in &all_payments {
            let entry = by_method.entry(p.payment_method.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += p.payment_amount.parse::<f64>().unwrap_or(0.0);
        }
        let payments_by_method: serde_json::Value = by_method.into_iter()
            .map(|(k, (count, total))| serde_json::json!({
                "method": k, "count": count, "total": format!("{:.2}", total)
            }))
            .collect();

        // Group by status
        let mut by_status: std::collections::HashMap<String, (i32, f64)> = std::collections::HashMap::new();
        for p in &all_payments {
            let entry = by_status.entry(p.status.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += p.payment_amount.parse::<f64>().unwrap_or(0.0);
        }
        let payments_by_status: serde_json::Value = by_status.into_iter()
            .map(|(k, (count, total))| serde_json::json!({
                "status": k, "count": count, "total": format!("{:.2}", total)
            }))
            .collect();

        Ok(PaymentDashboardSummary {
            total_pending_payment_count: total_pending_count,
            total_pending_payment_amount: format!("{:.2}", total_pending_amount),
            total_paid_payment_count: total_paid_count,
            total_paid_payment_amount: format!("{:.2}", total_paid_amount),
            total_discount_taken: format!("{:.2}", total_discount),
            payments_by_method,
            payments_by_status,
            upcoming_scheduled_count: upcoming_count,
            upcoming_scheduled_amount: format!("{:.2}", upcoming_amount),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_payment_methods() {
        assert!(VALID_PAYMENT_METHODS.contains(&"check"));
        assert!(VALID_PAYMENT_METHODS.contains(&"eft"));
        assert!(VALID_PAYMENT_METHODS.contains(&"wire"));
        assert!(VALID_PAYMENT_METHODS.contains(&"ach"));
    }

    #[test]
    fn test_valid_payment_statuses() {
        assert!(VALID_PAYMENT_STATUSES.contains(&"draft"));
        assert!(VALID_PAYMENT_STATUSES.contains(&"issued"));
        assert!(VALID_PAYMENT_STATUSES.contains(&"cleared"));
        assert!(VALID_PAYMENT_STATUSES.contains(&"voided"));
        assert!(VALID_PAYMENT_STATUSES.contains(&"reconciled"));
        assert!(VALID_PAYMENT_STATUSES.contains(&"stopped"));
    }

    #[test]
    fn test_valid_batch_statuses() {
        assert!(VALID_BATCH_STATUSES.contains(&"draft"));
        assert!(VALID_BATCH_STATUSES.contains(&"selected"));
        assert!(VALID_BATCH_STATUSES.contains(&"approved"));
        assert!(VALID_BATCH_STATUSES.contains(&"formatted"));
        assert!(VALID_BATCH_STATUSES.contains(&"confirmed"));
        assert!(VALID_BATCH_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_scheduled_statuses() {
        assert!(VALID_SCHEDULED_STATUSES.contains(&"pending"));
        assert!(VALID_SCHEDULED_STATUSES.contains(&"selected"));
        assert!(VALID_SCHEDULED_STATUSES.contains(&"paid"));
        assert!(VALID_SCHEDULED_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_frequencies() {
        assert!(VALID_FREQUENCIES.contains(&"monthly"));
        assert!(VALID_FREQUENCIES.contains(&"quarterly"));
        assert!(VALID_FREQUENCIES.contains(&"weekly"));
    }

    #[test]
    fn test_valid_delivery_methods() {
        assert!(VALID_DELIVERY_METHODS.contains(&"email"));
        assert!(VALID_DELIVERY_METHODS.contains(&"print"));
        assert!(VALID_DELIVERY_METHODS.contains(&"edi"));
        assert!(VALID_DELIVERY_METHODS.contains(&"xml"));
    }

    #[test]
    fn test_valid_format_types() {
        assert!(VALID_FORMAT_TYPES.contains(&"file"));
        assert!(VALID_FORMAT_TYPES.contains(&"printed_check"));
        assert!(VALID_FORMAT_TYPES.contains(&"edi"));
        assert!(VALID_FORMAT_TYPES.contains(&"xml"));
        assert!(VALID_FORMAT_TYPES.contains(&"json"));
    }

    #[test]
    fn test_calculate_due_date_net_30() {
        let engine = PaymentEngine::new(Arc::new(crate::MockPaymentRepository));
        let term = PaymentTerm {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: "NET30".to_string(),
            name: "Net 30".to_string(),
            description: None,
            due_days: 30,
            discount_days: None,
            discount_percentage: None,
            is_installment: false,
            installment_count: None,
            installment_frequency: None,
            default_payment_method: None,
            effective_from: None,
            effective_to: None,
            is_active: true,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let invoice_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let due_date = engine.calculate_due_date(&term, invoice_date);
        assert_eq!(due_date, chrono::NaiveDate::from_ymd_opt(2024, 2, 14).unwrap());
    }

    #[test]
    fn test_calculate_due_date_due_on_receipt() {
        let engine = PaymentEngine::new(Arc::new(crate::MockPaymentRepository));
        let term = PaymentTerm {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: "IMMEDIATE".to_string(),
            name: "Due on Receipt".to_string(),
            description: None,
            due_days: 0,
            discount_days: None,
            discount_percentage: None,
            is_installment: false,
            installment_count: None,
            installment_frequency: None,
            default_payment_method: None,
            effective_from: None,
            effective_to: None,
            is_active: true,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let invoice_date = chrono::NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let due_date = engine.calculate_due_date(&term, invoice_date);
        assert_eq!(due_date, invoice_date);
    }

    #[test]
    fn test_early_payment_discount_eligible() {
        let engine = PaymentEngine::new(Arc::new(crate::MockPaymentRepository));
        let term = PaymentTerm {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: "2_10_NET30".to_string(),
            name: "2% 10 Net 30".to_string(),
            description: None,
            due_days: 30,
            discount_days: Some(10),
            discount_percentage: Some("2.0".to_string()),
            is_installment: false,
            installment_count: None,
            installment_frequency: None,
            default_payment_method: None,
            effective_from: None,
            effective_to: None,
            is_active: true,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Paid within 10 days - should get discount
        let discount = engine.calculate_early_payment_discount(&term, "10000.00", 5);
        assert!(discount.is_some());
        let discount = discount.unwrap();
        assert!((discount.parse::<f64>().unwrap() - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_early_payment_discount_not_eligible() {
        let engine = PaymentEngine::new(Arc::new(crate::MockPaymentRepository));
        let term = PaymentTerm {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: "2_10_NET30".to_string(),
            name: "2% 10 Net 30".to_string(),
            description: None,
            due_days: 30,
            discount_days: Some(10),
            discount_percentage: Some("2.0".to_string()),
            is_installment: false,
            installment_count: None,
            installment_frequency: None,
            default_payment_method: None,
            effective_from: None,
            effective_to: None,
            is_active: true,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Paid after discount period - no discount
        let discount = engine.calculate_early_payment_discount(&term, "10000.00", 15);
        assert!(discount.is_none());
    }

    #[test]
    fn test_early_payment_discount_no_discount_term() {
        let engine = PaymentEngine::new(Arc::new(crate::MockPaymentRepository));
        let term = PaymentTerm {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: "NET30".to_string(),
            name: "Net 30".to_string(),
            description: None,
            due_days: 30,
            discount_days: None,
            discount_percentage: None,
            is_installment: false,
            installment_count: None,
            installment_frequency: None,
            default_payment_method: None,
            effective_from: None,
            effective_to: None,
            is_active: true,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Term has no discount - should return None
        let discount = engine.calculate_early_payment_discount(&term, "10000.00", 1);
        assert!(discount.is_none());
    }

    #[test]
    fn test_dashboard_summary_empty() {
        let engine = PaymentEngine::new(Arc::new(crate::MockPaymentRepository));

        // Use tokio runtime for async
        let rt = tokio::runtime::Runtime::new().unwrap();
        let summary = rt.block_on(async {
            engine.get_dashboard_summary(Uuid::new_v4()).await.unwrap()
        });

        assert_eq!(summary.total_pending_payment_count, 0);
        assert_eq!(summary.total_paid_payment_count, 0);
        assert_eq!(summary.upcoming_scheduled_count, 0);
    }
}
