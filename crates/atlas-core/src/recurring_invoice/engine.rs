//! Recurring Invoice Engine
//!
//! Orchestrates recurring AP invoice template management,
//! line management, invoice generation, and dashboard summaries.
//!
//! Oracle Fusion Cloud ERP: Financials > Payables > Recurring Invoices

use atlas_shared::{AtlasError, AtlasResult};
use super::repository::{
    RecurringInvoiceRepository,
    RecurringInvoiceTemplate, RecurringInvoiceTemplateLine,
    RecurringInvoiceGeneration,
    TemplateCreateParams, TemplateLineCreateParams,
};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// Valid values
const VALID_INVOICE_TYPES: &[&str] = &["standard", "credit_memo", "debit_memo", "prepayment"];
const VALID_AMOUNT_TYPES: &[&str] = &["fixed", "variable", "adjusted"];
const VALID_RECURRENCE_TYPES: &[&str] = &[
    "daily", "weekly", "monthly", "quarterly", "semi_annual", "annual",
];
const VALID_LINE_TYPES: &[&str] = &["item", "freight", "tax", "miscellaneous"];
const VALID_GL_DATE_BASIS: &[&str] = &["generation_date", "due_date", "period_end"];
const VALID_STATUSES: &[&str] = &["draft", "active", "suspended", "completed", "cancelled"];
const VALID_GENERATION_STATUSES: &[&str] = &[
    "generated", "submitted", "approved", "paid", "cancelled", "error",
];

/// Recurring Invoice Engine
pub struct RecurringInvoiceEngine {
    repo: Arc<dyn RecurringInvoiceRepository>,
}

impl RecurringInvoiceEngine {
    pub fn new(repo: Arc<dyn RecurringInvoiceRepository>) -> Self {
        Self { repo }
    }

    // ========================================================================
    // Template CRUD
    // ========================================================================

    /// Create a new recurring invoice template
    pub async fn create_template(
        &self,
        org_id: Uuid,
        template_number: &str,
        template_name: &str,
        description: Option<&str>,
        supplier_id: Option<Uuid>,
        supplier_number: Option<&str>,
        supplier_name: Option<&str>,
        supplier_site: Option<&str>,
        invoice_type: &str,
        invoice_currency_code: &str,
        payment_currency_code: Option<&str>,
        exchange_rate_type: Option<&str>,
        payment_terms: Option<&str>,
        payment_method: Option<&str>,
        payment_due_days: i32,
        liability_account_code: Option<&str>,
        expense_account_code: Option<&str>,
        amount_type: &str,
        recurrence_type: &str,
        recurrence_interval: i32,
        generation_day: Option<i32>,
        days_in_advance: i32,
        effective_from: chrono::NaiveDate,
        effective_to: Option<chrono::NaiveDate>,
        maximum_generations: Option<i32>,
        auto_submit: bool,
        auto_approve: bool,
        hold_for_review: bool,
        po_number: Option<&str>,
        gl_date_basis: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<RecurringInvoiceTemplate> {
        // Validate
        if template_number.is_empty() || template_name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Template number and name are required".to_string(),
            ));
        }
        if !VALID_INVOICE_TYPES.contains(&invoice_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid invoice_type '{}'. Must be one of: {}",
                invoice_type, VALID_INVOICE_TYPES.join(", ")
            )));
        }
        if !VALID_AMOUNT_TYPES.contains(&amount_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid amount_type '{}'. Must be one of: {}",
                amount_type, VALID_AMOUNT_TYPES.join(", ")
            )));
        }
        if !VALID_RECURRENCE_TYPES.contains(&recurrence_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid recurrence_type '{}'. Must be one of: {}",
                recurrence_type, VALID_RECURRENCE_TYPES.join(", ")
            )));
        }
        if !VALID_GL_DATE_BASIS.contains(&gl_date_basis) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid gl_date_basis '{}'. Must be one of: {}",
                gl_date_basis, VALID_GL_DATE_BASIS.join(", ")
            )));
        }
        if recurrence_interval < 1 {
            return Err(AtlasError::ValidationFailed(
                "Recurrence interval must be at least 1".to_string(),
            ));
        }
        if payment_due_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "Payment due days must be non-negative".to_string(),
            ));
        }
        if let Some(to) = effective_to {
            if to <= effective_from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".to_string(),
                ));
            }
        }
        if let Some(max) = maximum_generations {
            if max < 1 {
                return Err(AtlasError::ValidationFailed(
                    "Maximum generations must be at least 1".to_string(),
                ));
            }
        }

        let params = TemplateCreateParams {
            template_number: template_number.to_string(),
            template_name: template_name.to_string(),
            description: description.map(|s| s.to_string()),
            supplier_id,
            supplier_number: supplier_number.map(|s| s.to_string()),
            supplier_name: supplier_name.map(|s| s.to_string()),
            supplier_site: supplier_site.map(|s| s.to_string()),
            invoice_type: invoice_type.to_string(),
            invoice_currency_code: invoice_currency_code.to_string(),
            payment_currency_code: payment_currency_code.map(|s| s.to_string()),
            exchange_rate_type: exchange_rate_type.map(|s| s.to_string()),
            payment_terms: payment_terms.map(|s| s.to_string()),
            payment_method: payment_method.map(|s| s.to_string()),
            payment_due_days,
            liability_account_code: liability_account_code.map(|s| s.to_string()),
            expense_account_code: expense_account_code.map(|s| s.to_string()),
            amount_type: amount_type.to_string(),
            recurrence_type: recurrence_type.to_string(),
            recurrence_interval,
            generation_day,
            days_in_advance,
            effective_from,
            effective_to,
            maximum_generations,
            auto_submit,
            auto_approve,
            hold_for_review,
            po_number: po_number.map(|s| s.to_string()),
            gl_date_basis: gl_date_basis.to_string(),
        };

        info!("Recurring Invoice: Creating template '{}' for org {}", template_number, org_id);
        self.repo.create_template(org_id, &params, created_by).await
    }

    /// Get a template by ID
    pub async fn get_template(&self, id: Uuid) -> AtlasResult<Option<RecurringInvoiceTemplate>> {
        self.repo.get_template(id).await
    }

    /// Get a template by number
    pub async fn get_template_by_number(
        &self,
        org_id: Uuid,
        template_number: &str,
    ) -> AtlasResult<Option<RecurringInvoiceTemplate>> {
        self.repo.get_template_by_number(org_id, template_number).await
    }

    /// List templates with optional filters
    pub async fn list_templates(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        supplier_id: Option<Uuid>,
    ) -> AtlasResult<Vec<RecurringInvoiceTemplate>> {
        self.repo.list_templates(org_id, status, supplier_id).await
    }

    /// Transition template status
    /// draft → active, active → suspended, active → completed,
    /// suspended → active, draft → cancelled, active → cancelled
    pub async fn transition_template(
        &self,
        id: Uuid,
        new_status: &str,
    ) -> AtlasResult<RecurringInvoiceTemplate> {
        if !VALID_STATUSES.contains(&new_status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid status '{}'. Must be one of: {}",
                new_status, VALID_STATUSES.join(", ")
            )));
        }

        let template = self.repo.get_template(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Template not found".to_string()))?;

        Self::validate_status_transition(&template.status, new_status)?;

        // Calculate next generation date when activating
        let next_gen = if new_status == "active" {
            Some(Self::calculate_next_generation_date(
                template.last_generation_date.unwrap_or(chrono::Utc::now().naive_utc().date()),
                &template.recurrence_type,
                template.recurrence_interval,
            ))
        } else {
            None
        };

        info!("Recurring Invoice: Transitioning template {} from {} to {}",
            template.template_number, template.status, new_status);
        self.repo.update_template_status(id, new_status, next_gen).await
    }

    /// Delete a draft template
    pub async fn delete_template(
        &self,
        org_id: Uuid,
        template_number: &str,
    ) -> AtlasResult<()> {
        self.repo.delete_template(org_id, template_number).await
    }

    // ========================================================================
    // Template Lines
    // ========================================================================

    /// Add a line to a template
    pub async fn add_template_line(
        &self,
        org_id: Uuid,
        template_id: Uuid,
        line_type: &str,
        description: Option<&str>,
        item_code: Option<&str>,
        unit_of_measure: Option<&str>,
        amount: f64,
        quantity: f64,
        unit_price: Option<f64>,
        gl_account_code: &str,
        cost_center: Option<&str>,
        department: Option<&str>,
        tax_code: Option<&str>,
        tax_amount: f64,
        project_id: Option<Uuid>,
        expenditure_type: Option<&str>,
    ) -> AtlasResult<RecurringInvoiceTemplateLine> {
        // Validate template exists and is in draft status
        let template = self.repo.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Template not found".to_string()))?;

        if template.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Lines can only be added to draft templates".to_string(),
            ));
        }

        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line_type '{}'. Must be one of: {}",
                line_type, VALID_LINE_TYPES.join(", ")
            )));
        }

        if amount < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Amount cannot be negative".to_string(),
            ));
        }

        if gl_account_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "GL account code is required".to_string(),
            ));
        }

        // Get next line number
        let existing_lines = self.repo.list_template_lines(template_id).await?;
        let line_number = (existing_lines.len() as i32) + 1;

        let params = TemplateLineCreateParams {
            line_type: line_type.to_string(),
            description: description.map(|s| s.to_string()),
            item_code: item_code.map(|s| s.to_string()),
            unit_of_measure: unit_of_measure.map(|s| s.to_string()),
            amount: amount,
            quantity: quantity,
            unit_price: unit_price,
            gl_account_code: gl_account_code.to_string(),
            cost_center: cost_center.map(|s| s.to_string()),
            department: department.map(|s| s.to_string()),
            tax_code: tax_code.map(|s| s.to_string()),
            tax_amount: tax_amount,
            project_id,
            expenditure_type: expenditure_type.map(|s| s.to_string()),
        };

        info!("Recurring Invoice: Adding line to template {}", template.template_number);
        self.repo.create_template_line(org_id, template_id, line_number, &params).await
    }

    /// List lines for a template
    pub async fn list_template_lines(
        &self,
        template_id: Uuid,
    ) -> AtlasResult<Vec<RecurringInvoiceTemplateLine>> {
        self.repo.list_template_lines(template_id).await
    }

    /// Remove a line from a template
    pub async fn remove_template_line(
        &self,
        template_id: Uuid,
        line_id: Uuid,
    ) -> AtlasResult<()> {
        // Validate template is in draft
        let template = self.repo.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Template not found".to_string()))?;

        if template.status != "draft" {
            return Err(AtlasError::ValidationFailed(
                "Lines can only be removed from draft templates".to_string(),
            ));
        }

        self.repo.remove_template_line(template_id, line_id).await
    }

    // ========================================================================
    // Invoice Generation
    // ========================================================================

    /// Generate an invoice from a template
    pub async fn generate_invoice(
        &self,
        template_id: Uuid,
        invoice_date: chrono::NaiveDate,
        period_name: Option<&str>,
        fiscal_year: Option<i32>,
        period_number: Option<i32>,
        generated_by: Option<Uuid>,
    ) -> AtlasResult<RecurringInvoiceGeneration> {
        let template = self.repo.get_template(template_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Template not found".to_string()))?;

        if template.status != "active" {
            return Err(AtlasError::ValidationFailed(
                "Can only generate invoices from active templates".to_string(),
            ));
        }

        // Check effective date range
        if invoice_date < template.effective_from {
            return Err(AtlasError::ValidationFailed(
                "Invoice date is before template effective date".to_string(),
            ));
        }
        if let Some(to) = template.effective_to {
            if invoice_date > to {
                return Err(AtlasError::ValidationFailed(
                    "Invoice date is after template effective end date".to_string(),
                ));
            }
        }

        // Check maximum generations
        if let Some(max) = template.maximum_generations {
            if template.generation_count >= max {
                return Err(AtlasError::ValidationFailed(
                    "Maximum number of generations reached".to_string(),
                ));
            }
        }

        // Calculate amounts from template lines
        let lines = self.repo.list_template_lines(template_id).await?;
        let total_invoice_amount: f64 = lines.iter()
            .map(|l| l.amount)
            .sum();
        let total_tax_amount: f64 = lines.iter()
            .map(|l| l.tax_amount)
            .sum();
        let total_amount = total_invoice_amount + total_tax_amount;

        if total_amount <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Total invoice amount must be positive. Add lines to the template first.".to_string(),
            ));
        }

        // Calculate dates
        let invoice_due_date = invoice_date + chrono::Duration::days(template.payment_due_days as i64);
        let gl_date = match template.gl_date_basis.as_str() {
            "due_date" => invoice_due_date,
            "period_end" => invoice_date, // simplified
            _ => invoice_date, // generation_date
        };

        let generation_number = template.generation_count + 1;
        let next_gen = Self::calculate_next_generation_date(
            invoice_date,
            &template.recurrence_type,
            template.recurrence_interval,
        );

        // Create generation record
        let generation = self.repo.create_generation(
            template.organization_id,
            template_id,
            generation_number,
            invoice_date,
            invoice_due_date,
            gl_date,
            total_invoice_amount,
            total_tax_amount,
            total_amount,
            period_name,
            fiscal_year,
            period_number,
            generated_by,
        ).await?;

        // Update template
        self.repo.update_template_generation(
            template_id,
            invoice_date,
            Some(next_gen),
            total_amount,
        ).await?;

        info!(
            "Recurring Invoice: Generated invoice #{} for template '{}' (amount: {:.2})",
            generation_number, template.template_number, total_amount
        );

        Ok(generation)
    }

    /// List generation history
    pub async fn list_generations(
        &self,
        org_id: Uuid,
        template_id: Option<Uuid>,
        generation_status: Option<&str>,
    ) -> AtlasResult<Vec<RecurringInvoiceGeneration>> {
        self.repo.list_generations(org_id, template_id, generation_status).await
    }

    /// Update generation status
    pub async fn update_generation_status(
        &self,
        id: Uuid,
        generation_status: &str,
        error_message: Option<&str>,
    ) -> AtlasResult<RecurringInvoiceGeneration> {
        if !VALID_GENERATION_STATUSES.contains(&generation_status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid generation_status '{}'. Must be one of: {}",
                generation_status, VALID_GENERATION_STATUSES.join(", ")
            )));
        }
        self.repo.update_generation_status(id, generation_status, error_message).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the recurring invoice dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<super::repository::RecurringInvoiceDashboard> {
        self.repo.get_dashboard(org_id).await
    }

    // ========================================================================
    // Calculation Helpers (pure functions)
    // ========================================================================

    /// Validate that a status transition is allowed
    pub fn validate_status_transition(current: &str, target: &str) -> AtlasResult<()> {
        match (current, target) {
            ("draft", "active") => Ok(()),
            ("draft", "cancelled") => Ok(()),
            ("active", "suspended") => Ok(()),
            ("active", "completed") => Ok(()),
            ("active", "cancelled") => Ok(()),
            ("suspended", "active") => Ok(()),
            ("suspended", "cancelled") => Ok(()),
            ("completed", "cancelled") => Ok(()), // for audit/correction
            _ => Err(AtlasError::WorkflowError(format!(
                "Invalid status transition from '{}' to '{}'. \
                 Valid: draft→active, draft→cancelled, active→suspended, \
                 active→completed, active→cancelled, suspended→active, \
                 suspended→cancelled",
                current, target
            ))),
        }
    }

    /// Calculate the next generation date based on recurrence
    pub fn calculate_next_generation_date(
        current_date: chrono::NaiveDate,
        recurrence_type: &str,
        interval: i32,
    ) -> chrono::NaiveDate {
        let i = if interval < 1 { 1 } else { interval } as u32;
        match recurrence_type {
            "daily" => current_date + chrono::Duration::days(i as i64),
            "weekly" => current_date + chrono::Duration::weeks(i as i64),
            "monthly" => {
                let mut next = current_date;
                for _ in 0..i {
                    next = next.checked_add_months(chrono::Months::new(1))
                        .unwrap_or(next);
                }
                next
            }
            "quarterly" => {
                let mut next = current_date;
                for _ in 0..(i * 3) {
                    next = next.checked_add_months(chrono::Months::new(1))
                        .unwrap_or(next);
                }
                next
            }
            "semi_annual" => {
                let mut next = current_date;
                for _ in 0..(i * 6) {
                    next = next.checked_add_months(chrono::Months::new(1))
                        .unwrap_or(next);
                }
                next
            }
            "annual" => {
                let mut next = current_date;
                for _ in 0..(i * 12) {
                    next = next.checked_add_months(chrono::Months::new(1))
                        .unwrap_or(next);
                }
                next
            }
            _ => current_date,
        }
    }

    /// Calculate total template amount from lines
    pub fn calculate_template_total(
        lines: &[RecurringInvoiceTemplateLine],
    ) -> (f64, f64, f64) {
        let invoice_amount: f64 = lines.iter().map(|l| l.amount).sum();
        let tax_amount: f64 = lines.iter().map(|l| l.tax_amount).sum();
        let total = invoice_amount + tax_amount;
        (invoice_amount, tax_amount, total)
    }

    /// Calculate invoice due date from invoice date and payment terms
    pub fn calculate_due_date(
        invoice_date: chrono::NaiveDate,
        payment_due_days: i32,
    ) -> chrono::NaiveDate {
        invoice_date + chrono::Duration::days(payment_due_days as i64)
    }

    /// Check if a template is eligible for generation
    pub fn is_eligible_for_generation(
        template: &RecurringInvoiceTemplate,
        as_of_date: chrono::NaiveDate,
    ) -> Result<bool, String> {
        if template.status != "active" {
            return Err("Template must be active".to_string());
        }
        if as_of_date < template.effective_from {
            return Err("Current date is before effective date".to_string());
        }
        if let Some(to) = template.effective_to {
            if as_of_date > to {
                return Err("Current date is after effective end date".to_string());
            }
        }
        if let Some(max) = template.maximum_generations {
            if template.generation_count >= max {
                return Err("Maximum generations reached".to_string());
            }
        }
        Ok(true)
    }
}
