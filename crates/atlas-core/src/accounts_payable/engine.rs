//! Accounts Payable Engine Implementation
//!
//! Manages supplier invoices, invoice lines, distributions, holds,
//! payment processing, and AP aging reporting.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Payables > Invoices, Payments, Holds

use atlas_shared::{
    ApInvoice, ApInvoiceLine, ApInvoiceDistribution, ApInvoiceHold, ApPayment,
    ApAgingSummary, ApAgingBySupplier,
    AtlasError, AtlasResult,
};
use super::AccountsPayableRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid invoice types
const VALID_INVOICE_TYPES: &[&str] = &[
    "standard", "credit_memo", "debit_memo", "prepayment", "expense_report", "po_default",
];

/// Valid invoice statuses for transitions
const VALID_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "paid", "cancelled", "on_hold",
];

/// Valid line types
const VALID_LINE_TYPES: &[&str] = &[
    "item", "freight", "tax", "miscellaneous", "withholding",
];

/// Valid hold types
const VALID_HOLD_TYPES: &[&str] = &[
    "system", "manual", "matching", "approval", "variance", "budget",
];

/// Valid payment statuses
const VALID_PAYMENT_STATUSES: &[&str] = &[
    "draft", "submitted", "confirmed", "cancelled", "reversed",
];

/// Accounts Payable engine for managing supplier invoices, payments, and holds
pub struct AccountsPayableEngine {
    repository: Arc<dyn AccountsPayableRepository>,
}

impl AccountsPayableEngine {
    pub fn new(repository: Arc<dyn AccountsPayableRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Invoice Management
    // ========================================================================

    /// Create a new AP invoice
    pub async fn create_invoice(
        &self,
        org_id: Uuid,
        invoice_number: &str,
        invoice_date: chrono::NaiveDate,
        invoice_type: &str,
        description: Option<&str>,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_site: Option<&str>,
        invoice_currency_code: &str,
        payment_currency_code: &str,
        exchange_rate: Option<&str>,
        exchange_rate_type: Option<&str>,
        exchange_date: Option<chrono::NaiveDate>,
        invoice_amount: &str,
        tax_amount: &str,
        payment_terms: Option<&str>,
        payment_method: Option<&str>,
        payment_due_date: Option<chrono::NaiveDate>,
        discount_date: Option<chrono::NaiveDate>,
        gl_date: Option<chrono::NaiveDate>,
        po_number: Option<&str>,
        receipt_number: Option<&str>,
        source: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoice> {
        if invoice_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Invoice number is required".to_string(),
            ));
        }
        if !VALID_INVOICE_TYPES.contains(&invoice_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid invoice_type '{}'. Must be one of: {}", invoice_type, VALID_INVOICE_TYPES.join(", ")
            )));
        }

        let inv_amount: f64 = invoice_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "invoice_amount must be a valid number".to_string(),
        ))?;
        let tax: f64 = tax_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "tax_amount must be a valid number".to_string(),
        ))?;

        // Credit memos should have negative amounts
        if invoice_type == "credit_memo" && inv_amount > 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Credit memo invoice_amount must be negative".to_string(),
            ));
        }

        let total = inv_amount + tax;

        info!("Creating AP invoice '{}' for org {} supplier {}", invoice_number, org_id, supplier_id);

        self.repository.create_invoice(
            org_id, invoice_number, invoice_date, invoice_type,
            description, supplier_id, supplier_number, supplier_name, supplier_site,
            invoice_currency_code, payment_currency_code,
            exchange_rate, exchange_rate_type, exchange_date,
            &format!("{:.2}", inv_amount),
            &format!("{:.2}", tax),
            &format!("{:.2}", total),
            payment_terms, payment_method,
            payment_due_date, discount_date, gl_date,
            po_number, receipt_number, source,
            created_by,
        ).await
    }

    /// Get an invoice by ID
    pub async fn get_invoice(&self, id: Uuid) -> AtlasResult<Option<ApInvoice>> {
        self.repository.get_invoice(id).await
    }

    /// Get an invoice by number within an org
    pub async fn get_invoice_by_number(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<ApInvoice>> {
        self.repository.get_invoice_by_number(org_id, invoice_number).await
    }

    /// List invoices with optional filters
    pub async fn list_invoices(
        &self,
        org_id: Uuid,
        supplier_id: Option<Uuid>,
        status: Option<&str>,
        invoice_type: Option<&str>,
    ) -> AtlasResult<Vec<ApInvoice>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        if let Some(t) = invoice_type {
            if !VALID_INVOICE_TYPES.contains(&t) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid invoice_type '{}'. Must be one of: {}", t, VALID_INVOICE_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_invoices(org_id, supplier_id, status, invoice_type).await
    }

    // ========================================================================
    // Invoice Workflow
    // ========================================================================

    /// Submit an invoice for approval
    pub async fn submit_invoice(&self, invoice_id: Uuid) -> AtlasResult<ApInvoice> {
        let invoice = self.repository.get_invoice(invoice_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Invoice {} not found", invoice_id)
            ))?;

        if invoice.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit invoice in '{}' status. Must be 'draft'.", invoice.status)
            ));
        }

        // Check invoice has at least one line
        let lines = self.repository.list_lines(invoice_id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Invoice must have at least one line before submission".to_string(),
            ));
        }

        // Check for balanced distributions
        let distributions = self.repository.list_distributions(invoice_id).await?;
        if distributions.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Invoice must have at least one distribution before submission".to_string(),
            ));
        }

        let dist_total: f64 = distributions.iter()
            .filter(|d| d.distribution_type == "charge")
            .map(|d| d.amount.parse().unwrap_or(0.0))
            .sum();
        let inv_total: f64 = invoice.invoice_amount.parse().unwrap_or(0.0);

        if (dist_total - inv_total).abs() > 0.01 {
            return Err(AtlasError::ValidationFailed(
                format!("Distribution total ({:.2}) does not match invoice amount ({:.2})", dist_total, inv_total)
            ));
        }

        info!("Submitting AP invoice {}", invoice_id);
        self.repository.update_invoice_status(invoice_id, "submitted", None, None, None, None).await
    }

    /// Approve an invoice
    pub async fn approve_invoice(&self, invoice_id: Uuid, approved_by: Uuid) -> AtlasResult<ApInvoice> {
        let invoice = self.repository.get_invoice(invoice_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Invoice {} not found", invoice_id)
            ))?;

        if invoice.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve invoice in '{}' status. Must be 'submitted'.", invoice.status)
            ));
        }

        // Check for active holds
        let holds = self.repository.list_active_holds(invoice_id).await?;
        if !holds.is_empty() {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve invoice: {} active hold(s) must be released first", holds.len())
            ));
        }

        info!("Approving AP invoice {} by {}", invoice_id, approved_by);
        self.repository.update_invoice_status(invoice_id, "approved", Some(approved_by), None, None, None).await
    }

    /// Cancel an invoice
    pub async fn cancel_invoice(
        &self,
        invoice_id: Uuid,
        cancelled_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<ApInvoice> {
        let invoice = self.repository.get_invoice(invoice_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Invoice {} not found", invoice_id)
            ))?;

        if invoice.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                "Invoice is already cancelled".to_string(),
            ));
        }
        if invoice.status == "paid" {
            return Err(AtlasError::WorkflowError(
                "Cannot cancel a paid invoice".to_string(),
            ));
        }

        info!("Cancelling AP invoice {} by {}", invoice_id, cancelled_by);
        self.repository.update_invoice_status(
            invoice_id, "cancelled", None,
            Some(reason.unwrap_or("Cancelled")), Some(cancelled_by), None,
        ).await
    }

    /// Mark invoice as paid (typically called from payment processing)
    pub async fn mark_invoice_paid(&self, invoice_id: Uuid, amount_paid: &str) -> AtlasResult<ApInvoice> {
        let invoice = self.repository.get_invoice(invoice_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Invoice {} not found", invoice_id)
            ))?;

        if invoice.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot pay invoice in '{}' status. Must be 'approved'.", invoice.status)
            ));
        }

        info!("Marking AP invoice {} as paid (amount: {})", invoice_id, amount_paid);
        self.repository.update_invoice_paid(invoice_id, amount_paid).await
    }

    // ========================================================================
    // Invoice Line Management
    // ========================================================================

    /// Add a line to an invoice
    pub async fn add_line(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        line_type: &str,
        description: Option<&str>,
        amount: &str,
        unit_price: Option<&str>,
        quantity_invoiced: Option<&str>,
        unit_of_measure: Option<&str>,
        po_line_id: Option<Uuid>,
        po_line_number: Option<&str>,
        product_code: Option<&str>,
        tax_code: Option<&str>,
        tax_amount: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoiceLine> {
        let invoice = self.repository.get_invoice(invoice_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Invoice {} not found", invoice_id)
            ))?;

        if invoice.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only add lines to a draft invoice".to_string(),
            ));
        }

        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line_type '{}'. Must be one of: {}", line_type, VALID_LINE_TYPES.join(", ")
            )));
        }

        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;

        // Determine line number
        let existing_lines = self.repository.list_lines(invoice_id).await?;
        let line_number = (existing_lines.len() + 1) as i32;

        info!("Adding line {} to AP invoice {}", line_number, invoice_id);

        let line = self.repository.create_line(
            org_id, invoice_id, line_number, line_type,
            description, &format!("{:.2}", amount_val),
            unit_price, quantity_invoiced, unit_of_measure,
            po_line_id, po_line_number, product_code,
            tax_code, tax_amount, created_by,
        ).await?;

        // Recalculate invoice totals
        self.recalculate_invoice_totals(invoice_id).await?;

        Ok(line)
    }

    /// Delete an invoice line
    pub async fn delete_line(&self, invoice_id: Uuid, line_id: Uuid) -> AtlasResult<()> {
        let invoice = self.repository.get_invoice(invoice_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Invoice {} not found", invoice_id)
            ))?;

        if invoice.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only delete lines from a draft invoice".to_string(),
            ));
        }

        self.repository.delete_line(line_id).await?;
        self.recalculate_invoice_totals(invoice_id).await?;
        Ok(())
    }

    /// List all lines for an invoice
    pub async fn list_lines(&self, invoice_id: Uuid) -> AtlasResult<Vec<ApInvoiceLine>> {
        self.repository.list_lines(invoice_id).await
    }

    // ========================================================================
    // Invoice Distribution Management
    // ========================================================================

    /// Add a distribution to an invoice
    pub async fn add_distribution(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        invoice_line_id: Option<Uuid>,
        distribution_type: &str,
        account_combination: Option<&str>,
        description: Option<&str>,
        amount: &str,
        currency_code: &str,
        exchange_rate: Option<&str>,
        gl_account: Option<&str>,
        cost_center: Option<&str>,
        department: Option<&str>,
        project_id: Option<Uuid>,
        task_id: Option<Uuid>,
        expenditure_type: Option<&str>,
        tax_code: Option<&str>,
        tax_recoverable: bool,
        tax_recoverable_amount: Option<&str>,
        accounting_date: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoiceDistribution> {
        let invoice = self.repository.get_invoice(invoice_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Invoice {} not found", invoice_id)
            ))?;

        if invoice.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only add distributions to a draft invoice".to_string(),
            ));
        }

        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;

        // Determine distribution line number
        let existing_dists = self.repository.list_distributions(invoice_id).await?;
        let dist_line_number = (existing_dists.len() + 1) as i32;

        info!("Adding distribution {} to AP invoice {}", dist_line_number, invoice_id);

        self.repository.create_distribution(
            org_id, invoice_id, invoice_line_id, dist_line_number,
            distribution_type, account_combination, description,
            &format!("{:.2}", amount_val), None, currency_code,
            exchange_rate, gl_account, cost_center, department,
            project_id, task_id, expenditure_type, tax_code,
            tax_recoverable, tax_recoverable_amount,
            accounting_date, created_by,
        ).await
    }

    /// List distributions for an invoice
    pub async fn list_distributions(&self, invoice_id: Uuid) -> AtlasResult<Vec<ApInvoiceDistribution>> {
        self.repository.list_distributions(invoice_id).await
    }

    // ========================================================================
    // Invoice Hold Management
    // ========================================================================

    /// Apply a hold to an invoice
    pub async fn apply_hold(
        &self,
        org_id: Uuid,
        invoice_id: Uuid,
        hold_type: &str,
        hold_reason: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApInvoiceHold> {
        let _invoice = self.repository.get_invoice(invoice_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Invoice {} not found", invoice_id)
            ))?;

        if !VALID_HOLD_TYPES.contains(&hold_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid hold_type '{}'. Must be one of: {}", hold_type, VALID_HOLD_TYPES.join(", ")
            )));
        }
        if hold_reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Hold reason is required".to_string(),
            ));
        }

        info!("Applying {} hold to AP invoice {}", hold_type, invoice_id);

        let hold = self.repository.create_hold(
            org_id, invoice_id, hold_type, hold_reason, created_by,
        ).await?;

        // Set invoice status to on_hold if currently submitted or approved
        let invoice = self.repository.get_invoice(invoice_id).await?;
        if let Some(inv) = invoice {
            if inv.status == "submitted" || inv.status == "approved" {
                self.repository.update_invoice_status(
                    invoice_id, "on_hold", None, None, None, None,
                ).await?;
            }
        }

        Ok(hold)
    }

    /// Release a hold
    pub async fn release_hold(
        &self,
        hold_id: Uuid,
        released_by: Uuid,
        release_reason: Option<&str>,
    ) -> AtlasResult<ApInvoiceHold> {
        let hold = self.repository.get_hold(hold_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Hold {} not found", hold_id)
            ))?;

        if hold.hold_status != "active" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot release hold in '{}' status. Must be 'active'.", hold.hold_status)
            ));
        }

        info!("Releasing hold {} on AP invoice {}", hold_id, hold.invoice_id);

        let released = self.repository.update_hold_status(
            hold_id, "released", Some(released_by), release_reason,
        ).await?;

        // Check if invoice has any remaining active holds
        let active_holds = self.repository.list_active_holds(released.invoice_id).await?;
        if active_holds.is_empty() {
            // Restore to submitted status
            self.repository.update_invoice_status(
                released.invoice_id, "submitted", None, None, None, None,
            ).await?;
        }

        Ok(released)
    }

    /// List all holds for an invoice
    pub async fn list_holds(&self, invoice_id: Uuid) -> AtlasResult<Vec<ApInvoiceHold>> {
        self.repository.list_holds(invoice_id).await
    }

    // ========================================================================
    // Payment Processing
    // ========================================================================

    /// Create a payment for one or more invoices
    pub async fn create_payment(
        &self,
        org_id: Uuid,
        payment_number: &str,
        payment_date: chrono::NaiveDate,
        payment_method: &str,
        payment_currency_code: &str,
        payment_amount: &str,
        bank_account_id: Option<Uuid>,
        bank_account_name: Option<&str>,
        payment_document: Option<&str>,
        supplier_id: Uuid,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        invoice_ids: &[Uuid],
        created_by: Option<Uuid>,
    ) -> AtlasResult<ApPayment> {
        if payment_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Payment number is required".to_string(),
            ));
        }

        let pay_amount: f64 = payment_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "payment_amount must be a valid number".to_string(),
        ))?;
        if pay_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Payment amount must be positive".to_string(),
            ));
        }

        // Validate all invoices exist, belong to supplier, and are approved
        let mut total_due = 0.0_f64;
        for inv_id in invoice_ids {
            let inv = self.repository.get_invoice(*inv_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Invoice {} not found", inv_id)
                ))?;
            if inv.supplier_id != supplier_id {
                return Err(AtlasError::ValidationFailed(
                    format!("Invoice {} does not belong to supplier {}", inv_id, supplier_id)
                ));
            }
            if inv.status != "approved" {
                return Err(AtlasError::WorkflowError(
                    format!("Invoice {} is in '{}' status, must be 'approved' for payment", inv_id, inv.status)
                ));
            }
            let remaining: f64 = inv.amount_remaining.parse().unwrap_or(0.0);
            total_due += remaining;
        }

        if pay_amount > total_due + 0.01 {
            return Err(AtlasError::ValidationFailed(
                format!("Payment amount ({:.2}) exceeds total amount due ({:.2})", pay_amount, total_due)
            ));
        }

        info!("Creating AP payment '{}' for supplier {} amount {}", payment_number, supplier_id, payment_amount);

        let payment = self.repository.create_payment(
            org_id, payment_number, payment_date, payment_method,
            payment_currency_code, payment_amount,
            bank_account_id, bank_account_name, payment_document,
            supplier_id, supplier_number, supplier_name,
            &serde_json::json!(invoice_ids),
            created_by,
        ).await?;

        // Mark invoices as paid
        for inv_id in invoice_ids {
            let inv = self.repository.get_invoice(*inv_id).await?;
            if let Some(inv) = inv {
                let remaining: f64 = inv.amount_remaining.parse().unwrap_or(0.0);
                let paid_so_far: f64 = inv.amount_paid.parse().unwrap_or(0.0);
                let _new_paid = paid_so_far + remaining.min(pay_amount / invoice_ids.len() as f64);
                // Simple model: mark entire invoice as paid
                self.repository.update_invoice_paid(*inv_id, &format!("{:.2}", inv.total_amount.parse().unwrap_or(0.0))).await?;
            }
        }

        Ok(payment)
    }

    /// Get a payment by ID
    pub async fn get_payment(&self, id: Uuid) -> AtlasResult<Option<ApPayment>> {
        self.repository.get_payment(id).await
    }

    /// List payments with optional filters
    pub async fn list_payments(
        &self,
        org_id: Uuid,
        supplier_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ApPayment>> {
        if let Some(s) = status {
            if !VALID_PAYMENT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid payment status '{}'. Must be one of: {}", s, VALID_PAYMENT_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_payments(org_id, supplier_id, status).await
    }

    /// Confirm a payment
    pub async fn confirm_payment(&self, payment_id: Uuid, confirmed_by: Uuid) -> AtlasResult<ApPayment> {
        let payment = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payment {} not found", payment_id)
            ))?;

        if payment.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot confirm payment in '{}' status. Must be 'submitted'.", payment.status)
            ));
        }

        info!("Confirming AP payment {} by {}", payment_id, confirmed_by);
        self.repository.update_payment_status(payment_id, "confirmed", Some(confirmed_by), None).await
    }

    /// Cancel a payment
    pub async fn cancel_payment(&self, payment_id: Uuid, reason: Option<&str>) -> AtlasResult<ApPayment> {
        let payment = self.repository.get_payment(payment_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Payment {} not found", payment_id)
            ))?;

        if payment.status == "confirmed" || payment.status == "cancelled" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel payment in '{}' status.", payment.status)
            ));
        }

        info!("Cancelling AP payment {}", payment_id);
        self.repository.update_payment_status(payment_id, "cancelled", None, reason).await
    }

    // ========================================================================
    // AP Aging & Reporting
    // ========================================================================

    /// Get AP aging summary
    pub async fn get_aging_summary(
        &self,
        org_id: Uuid,
        as_of_date: chrono::NaiveDate,
    ) -> AtlasResult<ApAgingSummary> {
        let invoices = self.repository.list_invoices(org_id, None, None, None).await?;

        let mut current = 0.0_f64;
        let mut aging_1_30 = 0.0_f64;
        let mut aging_31_60 = 0.0_f64;
        let mut aging_61_90 = 0.0_f64;
        let mut aging_91_plus = 0.0_f64;
        let mut total_outstanding = 0.0_f64;
        let mut supplier_map: std::collections::HashMap<Uuid, ApAgingBySupplier> = std::collections::HashMap::new();
        let mut invoice_count = 0i32;

        for inv in &invoices {
            if inv.status == "cancelled" || inv.status == "draft" || inv.status == "paid" {
                continue;
            }

            let remaining: f64 = inv.amount_remaining.parse().unwrap_or(0.0);
            if remaining.abs() < 0.01 {
                continue;
            }

            total_outstanding += remaining;
            invoice_count += 1;

            // Calculate aging bucket based on due date
            let days_overdue = if let Some(due) = inv.payment_due_date {
                (as_of_date - due).num_days().max(0) as i32
            } else {
                0
            };

            match days_overdue {
                0 => current += remaining,
                1..=30 => aging_1_30 += remaining,
                31..=60 => aging_31_60 += remaining,
                61..=90 => aging_61_90 += remaining,
                _ => aging_91_plus += remaining,
            }

            // Per-supplier aggregation
            let entry = supplier_map.entry(inv.supplier_id).or_insert_with(|| ApAgingBySupplier {
                supplier_id: inv.supplier_id,
                supplier_name: inv.supplier_name.clone().unwrap_or_default(),
                supplier_number: inv.supplier_number.clone(),
                total_outstanding: "0.00".to_string(),
                current_amount: "0.00".to_string(),
                aging_1_30: "0.00".to_string(),
                aging_31_60: "0.00".to_string(),
                aging_61_90: "0.00".to_string(),
                aging_91_plus: "0.00".to_string(),
                invoice_count: 0,
            });
            entry.total_outstanding = format!("{:.2}", remaining + entry.total_outstanding.parse::<f64>().unwrap_or(0.0));
            entry.invoice_count += 1;
            match days_overdue {
                0 => entry.current_amount = format!("{:.2}", remaining + entry.current_amount.parse::<f64>().unwrap_or(0.0)),
                1..=30 => entry.aging_1_30 = format!("{:.2}", remaining + entry.aging_1_30.parse::<f64>().unwrap_or(0.0)),
                31..=60 => entry.aging_31_60 = format!("{:.2}", remaining + entry.aging_31_60.parse::<f64>().unwrap_or(0.0)),
                61..=90 => entry.aging_61_90 = format!("{:.2}", remaining + entry.aging_61_90.parse::<f64>().unwrap_or(0.0)),
                _ => entry.aging_91_plus = format!("{:.2}", remaining + entry.aging_91_plus.parse::<f64>().unwrap_or(0.0)),
            }
        }

        let by_supplier: Vec<ApAgingBySupplier> = supplier_map.into_values().collect();

        Ok(ApAgingSummary {
            organization_id: org_id,
            as_of_date,
            total_outstanding: format!("{:.2}", total_outstanding),
            current_amount: format!("{:.2}", current),
            aging_1_30: format!("{:.2}", aging_1_30),
            aging_31_60: format!("{:.2}", aging_31_60),
            aging_61_90: format!("{:.2}", aging_61_90),
            aging_91_plus: format!("{:.2}", aging_91_plus),
            supplier_count: by_supplier.len() as i32,
            invoice_count,
            by_supplier,
        })
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Recalculate invoice totals from lines
    async fn recalculate_invoice_totals(&self, invoice_id: Uuid) -> AtlasResult<()> {
        let lines = self.repository.list_lines(invoice_id).await?;

        let mut line_total: f64 = 0.0;
        let mut tax_total: f64 = 0.0;
        for line in &lines {
            let amt: f64 = line.amount.parse().unwrap_or(0.0);
            line_total += amt;
            if let Some(tax) = &line.tax_amount {
                tax_total += tax.parse().unwrap_or(0.0);
            }
        }

        self.repository.update_invoice_amounts(
            invoice_id,
            &format!("{:.2}", line_total),
            &format!("{:.2}", tax_total),
            &format!("{:.2}", line_total + tax_total),
        ).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_invoice_types() {
        assert!(VALID_INVOICE_TYPES.contains(&"standard"));
        assert!(VALID_INVOICE_TYPES.contains(&"credit_memo"));
        assert!(VALID_INVOICE_TYPES.contains(&"debit_memo"));
        assert!(VALID_INVOICE_TYPES.contains(&"prepayment"));
        assert!(VALID_INVOICE_TYPES.contains(&"expense_report"));
        assert!(VALID_INVOICE_TYPES.contains(&"po_default"));
    }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"submitted"));
        assert!(VALID_STATUSES.contains(&"approved"));
        assert!(VALID_STATUSES.contains(&"paid"));
        assert!(VALID_STATUSES.contains(&"cancelled"));
        assert!(VALID_STATUSES.contains(&"on_hold"));
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"item"));
        assert!(VALID_LINE_TYPES.contains(&"freight"));
        assert!(VALID_LINE_TYPES.contains(&"tax"));
        assert!(VALID_LINE_TYPES.contains(&"miscellaneous"));
        assert!(VALID_LINE_TYPES.contains(&"withholding"));
    }

    #[test]
    fn test_valid_hold_types() {
        assert!(VALID_HOLD_TYPES.contains(&"system"));
        assert!(VALID_HOLD_TYPES.contains(&"manual"));
        assert!(VALID_HOLD_TYPES.contains(&"matching"));
        assert!(VALID_HOLD_TYPES.contains(&"approval"));
        assert!(VALID_HOLD_TYPES.contains(&"variance"));
        assert!(VALID_HOLD_TYPES.contains(&"budget"));
    }

    #[test]
    fn test_valid_payment_statuses() {
        assert!(VALID_PAYMENT_STATUSES.contains(&"draft"));
        assert!(VALID_PAYMENT_STATUSES.contains(&"submitted"));
        assert!(VALID_PAYMENT_STATUSES.contains(&"confirmed"));
        assert!(VALID_PAYMENT_STATUSES.contains(&"cancelled"));
        assert!(VALID_PAYMENT_STATUSES.contains(&"reversed"));
    }
}
