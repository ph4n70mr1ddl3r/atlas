//! Project Billing Engine
//!
//! Manages bill rate schedules, billing configurations per project,
//! billing events (milestones), project invoice generation, and
//! retention management.
//!
//! Oracle Fusion Cloud equivalent: Project Management > Project Billing

use atlas_shared::{
    BillRateSchedule, BillRateLine, ProjectBillingConfig,
    BillingEvent, ProjectInvoiceHeader, ProjectInvoiceLine,
    ProjectBillingDashboard,
    AtlasError, AtlasResult,
};
use super::ProjectBillingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ============================================================================
// Valid enum constants
// ============================================================================

const VALID_SCHEDULE_TYPES: &[&str] = &["standard", "overtime", "holiday", "custom"];

const VALID_SCHEDULE_STATUSES: &[&str] = &["draft", "active", "inactive"];

const VALID_BILLING_METHODS: &[&str] = &[
    "time_and_materials", "fixed_price", "milestone", "cost_plus", "retention",
];

const VALID_INVOICE_FORMATS: &[&str] = &["detailed", "summary", "consolidated"];

const VALID_BILLING_CYCLES: &[&str] = &["weekly", "biweekly", "monthly", "milestone"];

const VALID_BILLING_CONFIG_STATUSES: &[&str] = &["draft", "active", "completed", "cancelled"];

const VALID_EVENT_TYPES: &[&str] = &[
    "milestone", "progress", "completion", "retention_release",
];

const VALID_EVENT_STATUSES: &[&str] = &[
    "planned", "ready", "invoiced", "partially_invoiced", "cancelled",
];

const VALID_INVOICE_TYPES: &[&str] = &[
    "progress", "milestone", "t_and_m", "retention_release", "debit_memo", "credit_memo",
];

const VALID_INVOICE_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "rejected", "posted", "cancelled",
];

const VALID_LINE_SOURCES: &[&str] = &[
    "expenditure_item", "billing_event", "retention", "manual",
];

const VALID_PAYMENT_STATUSES: &[&str] = &["unpaid", "partially_paid", "paid"];

/// Compute retention amount from a bill total
fn compute_retention(bill_amount: f64, retention_pct: f64, retention_cap: f64) -> f64 {
    let ret = bill_amount * (retention_pct / 100.0);
    if retention_cap > 0.0 {
        ret.min(retention_cap)
    } else {
        ret
    }
}

/// Project Billing Engine
pub struct ProjectBillingEngine {
    repository: Arc<dyn ProjectBillingRepository>,
}

impl ProjectBillingEngine {
    pub fn new(repository: Arc<dyn ProjectBillingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Bill Rate Schedules
    // ========================================================================

    /// Create a bill rate schedule
    pub async fn create_schedule(
        &self,
        org_id: Uuid,
        schedule_number: &str,
        name: &str,
        description: Option<&str>,
        schedule_type: &str,
        currency_code: &str,
        effective_start: chrono::NaiveDate,
        effective_end: Option<chrono::NaiveDate>,
        default_markup_pct: f64,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BillRateSchedule> {
        if schedule_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Schedule number is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Schedule name is required".to_string()));
        }
        if !VALID_SCHEDULE_TYPES.contains(&schedule_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid schedule_type '{}'. Must be one of: {}", schedule_type, VALID_SCHEDULE_TYPES.join(", ")
            )));
        }
        if let Some(end) = effective_end {
            if end < effective_start {
                return Err(AtlasError::ValidationFailed(
                    "effective_end must be >= effective_start".to_string(),
                ));
            }
        }
        if default_markup_pct < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "default_markup_pct must be >= 0".to_string(),
            ));
        }

        // Check for duplicate
        if self.repository.get_schedule_by_number(org_id, schedule_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Schedule '{}' already exists", schedule_number
            )));
        }

        info!("Creating bill rate schedule '{}' ({}) for org {}", schedule_number, name, org_id);
        self.repository.create_schedule(
            org_id, schedule_number, name, description,
            schedule_type, currency_code, effective_start, effective_end,
            default_markup_pct, created_by,
        ).await
    }

    /// Get a schedule by ID
    pub async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<BillRateSchedule>> {
        self.repository.get_schedule(id).await
    }

    /// Get a schedule by number
    pub async fn get_schedule_by_number(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<BillRateSchedule>> {
        self.repository.get_schedule_by_number(org_id, schedule_number).await
    }

    /// List schedules for an organization
    pub async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<BillRateSchedule>> {
        self.repository.list_schedules(org_id, status).await
    }

    /// Activate a schedule
    pub async fn activate_schedule(&self, id: Uuid) -> AtlasResult<BillRateSchedule> {
        info!("Activating schedule {}", id);
        self.repository.update_schedule_status(id, "active").await
    }

    /// Deactivate a schedule
    pub async fn deactivate_schedule(&self, id: Uuid) -> AtlasResult<BillRateSchedule> {
        info!("Deactivating schedule {}", id);
        self.repository.update_schedule_status(id, "inactive").await
    }

    /// Delete a schedule
    pub async fn delete_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()> {
        info!("Deleting schedule '{}' for org {}", schedule_number, org_id);
        self.repository.delete_schedule(org_id, schedule_number).await
    }

    // ========================================================================
    // Bill Rate Lines
    // ========================================================================

    /// Add a rate line to a schedule
    pub async fn add_rate_line(
        &self,
        org_id: Uuid,
        schedule_id: Uuid,
        role_name: &str,
        project_id: Option<Uuid>,
        bill_rate: f64,
        unit_of_measure: &str,
        effective_start: chrono::NaiveDate,
        effective_end: Option<chrono::NaiveDate>,
        markup_pct: Option<f64>,
    ) -> AtlasResult<BillRateLine> {
        if role_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Role name is required".to_string()));
        }
        if bill_rate < 0.0 {
            return Err(AtlasError::ValidationFailed("Bill rate must be >= 0".to_string()));
        }
        if let Some(end) = effective_end {
            if end < effective_start {
                return Err(AtlasError::ValidationFailed(
                    "effective_end must be >= effective_start".to_string(),
                ));
            }
        }
        if let Some(mp) = markup_pct {
            if mp < 0.0 {
                return Err(AtlasError::ValidationFailed("Markup pct must be >= 0".to_string()));
            }
        }

        // Verify schedule exists
        self.repository.get_schedule(schedule_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", schedule_id)))?;

        info!("Adding rate line for role '{}' to schedule {} [rate={}]", role_name, schedule_id, bill_rate);
        self.repository.create_rate_line(
            org_id, schedule_id, role_name, project_id,
            bill_rate, unit_of_measure, effective_start, effective_end, markup_pct,
        ).await
    }

    /// List rate lines for a schedule
    pub async fn list_rate_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<BillRateLine>> {
        self.repository.list_rate_lines(schedule_id).await
    }

    /// Find the applicable rate for a role on a given date
    pub async fn find_rate_for_role(
        &self,
        schedule_id: Uuid,
        role_name: &str,
        date: chrono::NaiveDate,
    ) -> AtlasResult<Option<BillRateLine>> {
        self.repository.find_rate_for_role(schedule_id, role_name, date).await
    }

    /// Delete a rate line
    pub async fn delete_rate_line(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.delete_rate_line(id).await
    }

    // ========================================================================
    // Project Billing Config
    // ========================================================================

    /// Create a billing configuration for a project
    #[allow(clippy::too_many_arguments)]
    pub async fn create_billing_config(
        &self,
        org_id: Uuid,
        project_id: Uuid,
        billing_method: &str,
        bill_rate_schedule_id: Option<Uuid>,
        contract_amount: f64,
        currency_code: &str,
        invoice_format: &str,
        billing_cycle: &str,
        payment_terms_days: i32,
        retention_pct: f64,
        retention_amount_cap: f64,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        customer_po_number: Option<&str>,
        contract_number: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectBillingConfig> {
        if !VALID_BILLING_METHODS.contains(&billing_method) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid billing_method '{}'. Must be one of: {}", billing_method, VALID_BILLING_METHODS.join(", ")
            )));
        }
        if !VALID_INVOICE_FORMATS.contains(&invoice_format) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid invoice_format '{}'. Must be one of: {}", invoice_format, VALID_INVOICE_FORMATS.join(", ")
            )));
        }
        if !VALID_BILLING_CYCLES.contains(&billing_cycle) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid billing_cycle '{}'. Must be one of: {}", billing_cycle, VALID_BILLING_CYCLES.join(", ")
            )));
        }
        if contract_amount < 0.0 {
            return Err(AtlasError::ValidationFailed("Contract amount must be >= 0".to_string()));
        }
        if payment_terms_days < 0 {
            return Err(AtlasError::ValidationFailed("Payment terms days must be >= 0".to_string()));
        }
        if retention_pct < 0.0 || retention_pct > 100.0 {
            return Err(AtlasError::ValidationFailed("Retention pct must be 0-100".to_string()));
        }
        if retention_amount_cap < 0.0 {
            return Err(AtlasError::ValidationFailed("Retention amount cap must be >= 0".to_string()));
        }

        // For T&M / cost-plus, a bill rate schedule is required
        if (billing_method == "time_and_materials" || billing_method == "cost_plus")
            && bill_rate_schedule_id.is_none()
        {
            return Err(AtlasError::ValidationFailed(format!(
                "bill_rate_schedule_id is required for billing_method '{}'", billing_method
            )));
        }

        // Verify schedule exists if provided
        if let Some(sid) = bill_rate_schedule_id {
            self.repository.get_schedule(sid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", sid)))?;
        }

        // Check for duplicate project config
        if self.repository.get_billing_config_by_project(org_id, project_id).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Billing config already exists for project {}", project_id
            )));
        }

        info!("Creating billing config for project {} [method={}, amount={}]",
              project_id, billing_method, contract_amount);

        self.repository.create_billing_config(
            org_id, project_id, billing_method, bill_rate_schedule_id,
            contract_amount, currency_code, invoice_format, billing_cycle,
            payment_terms_days, retention_pct, retention_amount_cap,
            customer_id, customer_name, customer_po_number, contract_number,
            created_by,
        ).await
    }

    /// Get billing config by ID
    pub async fn get_billing_config(&self, id: Uuid) -> AtlasResult<Option<ProjectBillingConfig>> {
        self.repository.get_billing_config(id).await
    }

    /// Get billing config by project
    pub async fn get_billing_config_by_project(&self, org_id: Uuid, project_id: Uuid) -> AtlasResult<Option<ProjectBillingConfig>> {
        self.repository.get_billing_config_by_project(org_id, project_id).await
    }

    /// List billing configs
    pub async fn list_billing_configs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ProjectBillingConfig>> {
        self.repository.list_billing_configs(org_id, status).await
    }

    /// Activate a billing config
    pub async fn activate_billing_config(&self, id: Uuid) -> AtlasResult<ProjectBillingConfig> {
        info!("Activating billing config {}", id);
        self.repository.update_billing_config_status(id, "active").await
    }

    /// Cancel a billing config
    pub async fn cancel_billing_config(&self, id: Uuid) -> AtlasResult<ProjectBillingConfig> {
        info!("Cancelling billing config {}", id);
        self.repository.update_billing_config_status(id, "cancelled").await
    }

    // ========================================================================
    // Billing Events
    // ========================================================================

    /// Create a billing event
    #[allow(clippy::too_many_arguments)]
    pub async fn create_billing_event(
        &self,
        org_id: Uuid,
        project_id: Uuid,
        event_number: &str,
        event_name: &str,
        description: Option<&str>,
        event_type: &str,
        billing_amount: f64,
        currency_code: &str,
        completion_pct: f64,
        planned_date: Option<chrono::NaiveDate>,
        task_id: Option<Uuid>,
        task_name: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<BillingEvent> {
        if event_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Event number is required".to_string()));
        }
        if event_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Event name is required".to_string()));
        }
        if !VALID_EVENT_TYPES.contains(&event_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid event_type '{}'. Must be one of: {}", event_type, VALID_EVENT_TYPES.join(", ")
            )));
        }
        if billing_amount < 0.0 {
            return Err(AtlasError::ValidationFailed("Billing amount must be >= 0".to_string()));
        }
        if !(0.0..=100.0).contains(&completion_pct) {
            return Err(AtlasError::ValidationFailed("Completion pct must be 0-100".to_string()));
        }

        // Check for duplicate
        if self.repository.get_billing_event_by_number(org_id, event_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Billing event '{}' already exists", event_number
            )));
        }

        info!("Creating billing event '{}' ({}) for project {} [type={}, amount={}]",
              event_number, event_name, project_id, event_type, billing_amount);

        self.repository.create_billing_event(
            org_id, project_id, event_number, event_name, description,
            event_type, billing_amount, currency_code, completion_pct,
            planned_date, task_id, task_name, created_by,
        ).await
    }

    /// Get a billing event by ID
    pub async fn get_billing_event(&self, id: Uuid) -> AtlasResult<Option<BillingEvent>> {
        self.repository.get_billing_event(id).await
    }

    /// Get a billing event by number
    pub async fn get_billing_event_by_number(&self, org_id: Uuid, event_number: &str) -> AtlasResult<Option<BillingEvent>> {
        self.repository.get_billing_event_by_number(org_id, event_number).await
    }

    /// List billing events with optional filters
    pub async fn list_billing_events(
        &self,
        org_id: Uuid,
        project_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<BillingEvent>> {
        self.repository.list_billing_events(org_id, project_id, status).await
    }

    /// Complete a billing event (mark as ready for invoicing)
    pub async fn complete_billing_event(
        &self,
        id: Uuid,
        actual_date: chrono::NaiveDate,
        completion_pct: f64,
    ) -> AtlasResult<BillingEvent> {
        if !(0.0..=100.0).contains(&completion_pct) {
            return Err(AtlasError::ValidationFailed("Completion pct must be 0-100".to_string()));
        }

        // Verify event exists and is in a completable state
        let event = self.repository.get_billing_event(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Billing event {} not found", id)))?;

        if event.status != "planned" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot complete event in '{}' status. Must be 'planned'", event.status
            )));
        }

        info!("Completing billing event {} [pct={}]", id, completion_pct);
        self.repository.complete_billing_event(id, actual_date, completion_pct).await
    }

    /// Cancel a billing event
    pub async fn cancel_billing_event(&self, id: Uuid) -> AtlasResult<BillingEvent> {
        info!("Cancelling billing event {}", id);
        self.repository.update_billing_event_status(id, "cancelled").await
    }

    /// Delete a billing event
    pub async fn delete_billing_event(&self, org_id: Uuid, event_number: &str) -> AtlasResult<()> {
        info!("Deleting billing event '{}' for org {}", event_number, org_id);
        self.repository.delete_billing_event(org_id, event_number).await
    }

    // ========================================================================
    // Project Invoices
    // ========================================================================

    /// Create a draft invoice for a project.
    /// If a billing event is provided, creates a milestone/progress invoice.
    /// Otherwise, creates a T&M invoice.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_invoice(
        &self,
        org_id: Uuid,
        invoice_number: &str,
        project_id: Uuid,
        project_number: Option<&str>,
        project_name: Option<&str>,
        invoice_type: &str,
        customer_id: Option<Uuid>,
        customer_name: Option<&str>,
        billing_event_id: Option<Uuid>,
        lines: Vec<InvoiceLineRequest>,
        customer_po_number: Option<&str>,
        contract_number: Option<&str>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ProjectInvoiceHeader> {
        if invoice_number.is_empty() {
            return Err(AtlasError::ValidationFailed("Invoice number is required".to_string()));
        }
        if !VALID_INVOICE_TYPES.contains(&invoice_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid invoice_type '{}'. Must be one of: {}", invoice_type, VALID_INVOICE_TYPES.join(", ")
            )));
        }
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed("Invoice must have at least one line".to_string()));
        }

        // Check for duplicate invoice number
        if self.repository.get_invoice_by_number(org_id, invoice_number).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Invoice '{}' already exists", invoice_number
            )));
        }

        // Resolve billing config for retention calculation
        let config = self.repository.get_billing_config_by_project(org_id, project_id).await?;
        let retention_pct = config.as_ref().map(|c| c.retention_pct).unwrap_or(0.0);
        let retention_cap = config.as_ref().map(|c| c.retention_amount_cap).unwrap_or(0.0);
        let currency_code = config.as_ref().map(|c| c.currency_code.clone()).unwrap_or_else(|| "USD".to_string());
        let payment_terms = config.as_ref().map(|c| c.payment_terms_days).unwrap_or(30);

        // Calculate totals from lines
        let mut invoice_amount = 0.0_f64;
        let mut total_markup = 0.0_f64;
        let mut total_retention = 0.0_f64;
        let mut total_tax = 0.0_f64;

        for line in &lines {
            if line.bill_amount < 0.0 {
                return Err(AtlasError::ValidationFailed("Line bill amount must be >= 0".to_string()));
            }
            let line_retention = compute_retention(line.bill_amount, retention_pct, retention_cap);
            invoice_amount += line.bill_amount;
            total_markup += line.markup_amount;
            total_retention += line_retention;
            total_tax += line.tax_amount;
        }

        let total_amount = invoice_amount + total_tax - total_retention;

        let today = chrono::Utc::now().date_naive();
        let due_date = today + chrono::Duration::days(payment_terms as i64);

        info!("Creating invoice '{}' for project {} [type={}, amount={}, retention={}]",
              invoice_number, project_id, invoice_type, total_amount, total_retention);

        // Create the header
        let header = self.repository.create_invoice(
            org_id, invoice_number, project_id, project_number, project_name,
            invoice_type, customer_id, customer_name,
            invoice_amount, total_tax, total_retention, total_amount,
            &currency_code,
            None, None, // billing period – could be derived from lines
            today, Some(due_date),
            billing_event_id, customer_po_number, contract_number,
            notes, created_by,
        ).await?;

        // Create the lines
        for (i, line) in lines.iter().enumerate() {
            let line_retention = compute_retention(line.bill_amount, retention_pct, retention_cap);
            self.repository.create_invoice_line(
                org_id, header.id, (i + 1) as i32,
                &line.line_source, line.expenditure_item_id, line.billing_event_id,
                line.task_id, line.task_number.as_deref(), line.task_name.as_deref(),
                line.description.as_deref(),
                line.employee_id, line.employee_name.as_deref(),
                line.role_name.as_deref(), line.expenditure_type.as_deref(),
                line.quantity, &line.unit_of_measure, line.bill_rate,
                line.raw_cost_amount, line.bill_amount, line.markup_amount,
                line_retention, line.tax_amount,
                line.transaction_date,
            ).await?;
        }

        Ok(header)
    }

    /// Get an invoice by ID
    pub async fn get_invoice(&self, id: Uuid) -> AtlasResult<Option<ProjectInvoiceHeader>> {
        self.repository.get_invoice(id).await
    }

    /// Get an invoice by number
    pub async fn get_invoice_by_number(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<ProjectInvoiceHeader>> {
        self.repository.get_invoice_by_number(org_id, invoice_number).await
    }

    /// List invoices with optional filters
    pub async fn list_invoices(
        &self,
        org_id: Uuid,
        project_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ProjectInvoiceHeader>> {
        self.repository.list_invoices(org_id, project_id, status).await
    }

    /// Get invoice lines
    pub async fn get_invoice_lines(&self, invoice_header_id: Uuid) -> AtlasResult<Vec<ProjectInvoiceLine>> {
        self.repository.list_invoice_lines(invoice_header_id).await
    }

    /// Submit an invoice for approval
    pub async fn submit_invoice(&self, id: Uuid) -> AtlasResult<ProjectInvoiceHeader> {
        let invoice = self.repository.get_invoice(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;

        if invoice.status != "draft" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot submit invoice in '{}' status. Must be 'draft'", invoice.status
            )));
        }

        info!("Submitting invoice {} for approval", id);
        self.repository.update_invoice_status(id, "submitted").await
    }

    /// Approve an invoice
    pub async fn approve_invoice(&self, id: Uuid) -> AtlasResult<ProjectInvoiceHeader> {
        let invoice = self.repository.get_invoice(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;

        if invoice.status != "submitted" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot approve invoice in '{}' status. Must be 'submitted'", invoice.status
            )));
        }

        info!("Approving invoice {}", id);
        self.repository.update_invoice_status(id, "approved").await
    }

    /// Reject an invoice
    pub async fn reject_invoice(&self, id: Uuid, reason: &str) -> AtlasResult<ProjectInvoiceHeader> {
        let invoice = self.repository.get_invoice(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;

        if invoice.status != "submitted" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot reject invoice in '{}' status. Must be 'submitted'", invoice.status
            )));
        }

        info!("Rejecting invoice {} [reason: {}]", id, reason);
        self.repository.reject_invoice(id, reason).await
    }

    /// Post an invoice to GL
    pub async fn post_invoice(&self, id: Uuid) -> AtlasResult<ProjectInvoiceHeader> {
        let invoice = self.repository.get_invoice(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;

        if invoice.status != "approved" {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot post invoice in '{}' status. Must be 'approved'", invoice.status
            )));
        }

        info!("Posting invoice {} to GL", id);
        self.repository.mark_invoice_posted(id).await
    }

    /// Cancel an invoice
    pub async fn cancel_invoice(&self, id: Uuid) -> AtlasResult<ProjectInvoiceHeader> {
        let invoice = self.repository.get_invoice(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;

        if invoice.status == "posted" {
            return Err(AtlasError::ValidationFailed(
                "Cannot cancel a posted invoice. Create a credit memo instead.".to_string(),
            ));
        }

        info!("Cancelling invoice {}", id);
        self.repository.update_invoice_status(id, "cancelled").await
    }

    /// Delete an invoice (only drafts)
    pub async fn delete_invoice(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<()> {
        info!("Deleting invoice '{}' for org {}", invoice_number, org_id);
        self.repository.delete_invoice(org_id, invoice_number).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the project billing dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ProjectBillingDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

/// Request struct for creating an invoice line
#[derive(Debug, Clone)]
pub struct InvoiceLineRequest {
    pub line_source: String,
    pub expenditure_item_id: Option<Uuid>,
    pub billing_event_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub task_number: Option<String>,
    pub task_name: Option<String>,
    pub description: Option<String>,
    pub employee_id: Option<Uuid>,
    pub employee_name: Option<String>,
    pub role_name: Option<String>,
    pub expenditure_type: Option<String>,
    pub quantity: f64,
    pub unit_of_measure: String,
    pub bill_rate: f64,
    pub raw_cost_amount: f64,
    pub bill_amount: f64,
    pub markup_amount: f64,
    pub tax_amount: f64,
    pub transaction_date: Option<chrono::NaiveDate>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ========================================================================
    // Mock State & Repository
    // ========================================================================

    struct MockState {
        schedules: HashMap<Uuid, (String, BillRateSchedule)>, // (status, schedule)
        rate_lines: HashMap<Uuid, BillRateLine>,
        billing_configs: HashMap<Uuid, (String, ProjectBillingConfig)>,
        events: HashMap<Uuid, (String, BillingEvent)>,
        invoices: HashMap<Uuid, (String, ProjectInvoiceHeader)>,
        invoice_lines: HashMap<Uuid, ProjectInvoiceLine>,
        next_counter: u64,
    }

    struct MockBillingRepo {
        state: Arc<Mutex<MockState>>,
    }

    impl MockBillingRepo {
        fn new() -> Self {
            Self {
                state: Arc::new(Mutex::new(MockState {
                    schedules: HashMap::new(),
                    rate_lines: HashMap::new(),
                    billing_configs: HashMap::new(),
                    events: HashMap::new(),
                    invoices: HashMap::new(),
                    invoice_lines: HashMap::new(),
                    next_counter: 1,
                })),
            }
        }

        fn cloned(&self) -> Self {
            Self { state: self.state.clone() }
        }

        fn into_repo(self) -> Arc<dyn ProjectBillingRepository> {
            Arc::new(self)
        }

        fn next_id(&self) -> Uuid {
            let mut state = self.state.lock().unwrap();
            let id = state.next_counter;
            state.next_counter += 1;
            Uuid::from_u128(id as u128)
        }
    }

    fn make_schedule(id: Uuid, org_id: Uuid, number: &str, status: &str) -> BillRateSchedule {
        BillRateSchedule {
            id,
            organization_id: org_id,
            schedule_number: number.to_string(),
            name: format!("Schedule {}", number),
            description: String::new(),
            schedule_type: "standard".to_string(),
            currency_code: "USD".to_string(),
            effective_start: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            effective_end: None,
            status: status.to_string(),
            default_markup_pct: 0.0,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn make_billing_config(id: Uuid, org_id: Uuid, project_id: Uuid, method: &str, status: &str) -> ProjectBillingConfig {
        ProjectBillingConfig {
            id,
            organization_id: org_id,
            project_id,
            billing_method: method.to_string(),
            bill_rate_schedule_id: None,
            contract_amount: 100000.0,
            currency_code: "USD".to_string(),
            invoice_format: "detailed".to_string(),
            billing_cycle: "monthly".to_string(),
            payment_terms_days: 30,
            retention_pct: 10.0,
            retention_amount_cap: 0.0,
            customer_id: None,
            customer_name: "Test Customer".to_string(),
            customer_po_number: String::new(),
            contract_number: String::new(),
            status: status.to_string(),
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn make_event(id: Uuid, org_id: Uuid, project_id: Uuid, number: &str, status: &str) -> BillingEvent {
        BillingEvent {
            id,
            organization_id: org_id,
            project_id,
            event_number: number.to_string(),
            event_name: format!("Event {}", number),
            description: String::new(),
            event_type: "milestone".to_string(),
            billing_amount: 25000.0,
            currency_code: "USD".to_string(),
            completion_pct: 0.0,
            status: status.to_string(),
            planned_date: None,
            actual_date: None,
            task_id: None,
            task_name: String::new(),
            invoice_header_id: None,
            approved_by: None,
            approved_at: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn make_invoice(id: Uuid, org_id: Uuid, project_id: Uuid, number: &str, status: &str) -> ProjectInvoiceHeader {
        ProjectInvoiceHeader {
            id,
            organization_id: org_id,
            invoice_number: number.to_string(),
            project_id,
            project_number: "PRJ-001".to_string(),
            project_name: "Test Project".to_string(),
            invoice_type: "t_and_m".to_string(),
            status: status.to_string(),
            customer_id: None,
            customer_name: "Test Customer".to_string(),
            invoice_amount: 10000.0,
            tax_amount: 0.0,
            retention_held: 0.0,
            total_amount: 10000.0,
            currency_code: "USD".to_string(),
            exchange_rate: 1.0,
            billing_period_start: None,
            billing_period_end: None,
            invoice_date: chrono::Utc::now().date_naive(),
            due_date: None,
            billing_event_id: None,
            customer_po_number: String::new(),
            contract_number: String::new(),
            gl_posted_flag: false,
            gl_posted_date: None,
            approved_by: None,
            approved_at: None,
            rejected_reason: String::new(),
            payment_status: "unpaid".to_string(),
            payment_date: None,
            notes: String::new(),
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[async_trait::async_trait]
    impl ProjectBillingRepository for MockBillingRepo {
        // Schedules
        async fn create_schedule(
            &self, org_id: Uuid, schedule_number: &str, name: &str,
            description: Option<&str>, schedule_type: &str, currency_code: &str,
            effective_start: chrono::NaiveDate, effective_end: Option<chrono::NaiveDate>,
            default_markup_pct: f64, created_by: Option<Uuid>,
        ) -> AtlasResult<BillRateSchedule> {
            let id = self.next_id();
            let schedule = BillRateSchedule {
                id,
                organization_id: org_id,
                schedule_number: schedule_number.to_string(),
                name: name.to_string(),
                description: description.unwrap_or("").to_string(),
                schedule_type: schedule_type.to_string(),
                currency_code: currency_code.to_string(),
                effective_start,
                effective_end,
                status: "draft".to_string(),
                default_markup_pct,
                metadata: serde_json::json!({}),
                created_by,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let mut state = self.state.lock().unwrap();
            state.schedules.insert(id, ("draft".to_string(), schedule.clone()));
            Ok(schedule)
        }

        async fn get_schedule(&self, id: Uuid) -> AtlasResult<Option<BillRateSchedule>> {
            let state = self.state.lock().unwrap();
            Ok(state.schedules.get(&id).map(|(_, s)| s.clone()))
        }

        async fn get_schedule_by_number(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<Option<BillRateSchedule>> {
            let state = self.state.lock().unwrap();
            for (_, (_, s)) in &state.schedules {
                if s.organization_id == org_id && s.schedule_number == schedule_number {
                    return Ok(Some(s.clone()));
                }
            }
            Ok(None)
        }

        async fn list_schedules(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<BillRateSchedule>> {
            let state = self.state.lock().unwrap();
            let mut result = Vec::new();
            for (_, (st, s)) in &state.schedules {
                if s.organization_id == org_id && (status.is_none() || st == status.unwrap()) {
                    result.push(s.clone());
                }
            }
            Ok(result)
        }

        async fn update_schedule_status(&self, id: Uuid, status: &str) -> AtlasResult<BillRateSchedule> {
            let mut state = self.state.lock().unwrap();
            let entry = state.schedules.get_mut(&id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Schedule {} not found", id)))?;
            entry.0 = status.to_string();
            entry.1.status = status.to_string();
            entry.1.updated_at = chrono::Utc::now();
            Ok(entry.1.clone())
        }

        async fn delete_schedule(&self, org_id: Uuid, schedule_number: &str) -> AtlasResult<()> {
            let mut state = self.state.lock().unwrap();
            let to_remove: Vec<Uuid> = state.schedules.iter()
                .filter(|(_, (_, s))| s.organization_id == org_id && s.schedule_number == schedule_number)
                .map(|(id, _)| *id)
                .collect();
            if to_remove.is_empty() {
                return Err(AtlasError::EntityNotFound(format!("Schedule '{}' not found", schedule_number)));
            }
            for id in to_remove {
                state.schedules.remove(&id);
            }
            Ok(())
        }

        // Rate Lines
        async fn create_rate_line(
            &self, org_id: Uuid, schedule_id: Uuid, role_name: &str,
            project_id: Option<Uuid>, bill_rate: f64, unit_of_measure: &str,
            effective_start: chrono::NaiveDate, effective_end: Option<chrono::NaiveDate>,
            markup_pct: Option<f64>,
        ) -> AtlasResult<BillRateLine> {
            let id = self.next_id();
            let line = BillRateLine {
                id,
                organization_id: org_id,
                schedule_id,
                role_name: role_name.to_string(),
                project_id,
                bill_rate,
                unit_of_measure: unit_of_measure.to_string(),
                effective_start,
                effective_end,
                markup_pct,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let mut state = self.state.lock().unwrap();
            state.rate_lines.insert(id, line.clone());
            Ok(line)
        }

        async fn list_rate_lines(&self, schedule_id: Uuid) -> AtlasResult<Vec<BillRateLine>> {
            let state = self.state.lock().unwrap();
            let result: Vec<BillRateLine> = state.rate_lines.values()
                .filter(|l| l.schedule_id == schedule_id)
                .cloned()
                .collect();
            Ok(result)
        }

        async fn find_rate_for_role(
            &self, schedule_id: Uuid, role_name: &str, date: chrono::NaiveDate,
        ) -> AtlasResult<Option<BillRateLine>> {
            let state = self.state.lock().unwrap();
            for line in state.rate_lines.values() {
                if line.schedule_id == schedule_id && line.role_name == role_name
                    && line.effective_start <= date
                    && (line.effective_end.is_none() || line.effective_end.unwrap() >= date)
                {
                    return Ok(Some(line.clone()));
                }
            }
            Ok(None)
        }

        async fn delete_rate_line(&self, id: Uuid) -> AtlasResult<()> {
            let mut state = self.state.lock().unwrap();
            if state.rate_lines.remove(&id).is_none() {
                return Err(AtlasError::EntityNotFound("Rate line not found".to_string()));
            }
            Ok(())
        }

        // Billing Config
        async fn create_billing_config(
            &self, org_id: Uuid, project_id: Uuid, billing_method: &str,
            bill_rate_schedule_id: Option<Uuid>, contract_amount: f64,
            currency_code: &str, invoice_format: &str, billing_cycle: &str,
            payment_terms_days: i32, retention_pct: f64, retention_amount_cap: f64,
            customer_id: Option<Uuid>, customer_name: Option<&str>,
            customer_po_number: Option<&str>, contract_number: Option<&str>,
            created_by: Option<Uuid>,
        ) -> AtlasResult<ProjectBillingConfig> {
            let id = self.next_id();
            let config = ProjectBillingConfig {
                id,
                organization_id: org_id,
                project_id,
                billing_method: billing_method.to_string(),
                bill_rate_schedule_id,
                contract_amount,
                currency_code: currency_code.to_string(),
                invoice_format: invoice_format.to_string(),
                billing_cycle: billing_cycle.to_string(),
                payment_terms_days,
                retention_pct,
                retention_amount_cap,
                customer_id,
                customer_name: customer_name.unwrap_or("").to_string(),
                customer_po_number: customer_po_number.unwrap_or("").to_string(),
                contract_number: contract_number.unwrap_or("").to_string(),
                status: "draft".to_string(),
                metadata: serde_json::json!({}),
                created_by,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let mut state = self.state.lock().unwrap();
            state.billing_configs.insert(id, ("draft".to_string(), config.clone()));
            Ok(config)
        }

        async fn get_billing_config(&self, id: Uuid) -> AtlasResult<Option<ProjectBillingConfig>> {
            let state = self.state.lock().unwrap();
            Ok(state.billing_configs.get(&id).map(|(_, c)| c.clone()))
        }

        async fn get_billing_config_by_project(&self, org_id: Uuid, project_id: Uuid) -> AtlasResult<Option<ProjectBillingConfig>> {
            let state = self.state.lock().unwrap();
            for (_, (_, c)) in &state.billing_configs {
                if c.organization_id == org_id && c.project_id == project_id {
                    return Ok(Some(c.clone()));
                }
            }
            Ok(None)
        }

        async fn update_billing_config_status(&self, id: Uuid, status: &str) -> AtlasResult<ProjectBillingConfig> {
            let mut state = self.state.lock().unwrap();
            let entry = state.billing_configs.get_mut(&id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Config {} not found", id)))?;
            entry.0 = status.to_string();
            entry.1.status = status.to_string();
            entry.1.updated_at = chrono::Utc::now();
            Ok(entry.1.clone())
        }

        async fn list_billing_configs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<ProjectBillingConfig>> {
            let state = self.state.lock().unwrap();
            let mut result = Vec::new();
            for (_, (st, c)) in &state.billing_configs {
                if c.organization_id == org_id && (status.is_none() || st == status.unwrap()) {
                    result.push(c.clone());
                }
            }
            Ok(result)
        }

        // Billing Events
        async fn create_billing_event(
            &self, org_id: Uuid, project_id: Uuid, event_number: &str,
            event_name: &str, description: Option<&str>, event_type: &str,
            billing_amount: f64, currency_code: &str, completion_pct: f64,
            planned_date: Option<chrono::NaiveDate>,
            task_id: Option<Uuid>, task_name: Option<&str>,
            created_by: Option<Uuid>,
        ) -> AtlasResult<BillingEvent> {
            let id = self.next_id();
            let event = BillingEvent {
                id,
                organization_id: org_id,
                project_id,
                event_number: event_number.to_string(),
                event_name: event_name.to_string(),
                description: description.unwrap_or("").to_string(),
                event_type: event_type.to_string(),
                billing_amount,
                currency_code: currency_code.to_string(),
                completion_pct,
                status: "planned".to_string(),
                planned_date,
                actual_date: None,
                task_id,
                task_name: task_name.unwrap_or("").to_string(),
                invoice_header_id: None,
                approved_by: None,
                approved_at: None,
                metadata: serde_json::json!({}),
                created_by,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let mut state = self.state.lock().unwrap();
            state.events.insert(id, ("planned".to_string(), event.clone()));
            Ok(event)
        }

        async fn get_billing_event(&self, id: Uuid) -> AtlasResult<Option<BillingEvent>> {
            let state = self.state.lock().unwrap();
            Ok(state.events.get(&id).map(|(_, e)| e.clone()))
        }

        async fn get_billing_event_by_number(&self, org_id: Uuid, event_number: &str) -> AtlasResult<Option<BillingEvent>> {
            let state = self.state.lock().unwrap();
            for (_, (_, e)) in &state.events {
                if e.organization_id == org_id && e.event_number == event_number {
                    return Ok(Some(e.clone()));
                }
            }
            Ok(None)
        }

        async fn list_billing_events(
            &self, org_id: Uuid, project_id: Option<Uuid>, status: Option<&str>,
        ) -> AtlasResult<Vec<BillingEvent>> {
            let state = self.state.lock().unwrap();
            let mut result = Vec::new();
            for (_, (st, e)) in &state.events {
                if e.organization_id == org_id
                    && (project_id.is_none() || e.project_id == project_id.unwrap())
                    && (status.is_none() || st == status.unwrap())
                {
                    result.push(e.clone());
                }
            }
            Ok(result)
        }

        async fn update_billing_event_status(&self, id: Uuid, status: &str) -> AtlasResult<BillingEvent> {
            let mut state = self.state.lock().unwrap();
            let entry = state.events.get_mut(&id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Event {} not found", id)))?;
            entry.0 = status.to_string();
            entry.1.status = status.to_string();
            entry.1.updated_at = chrono::Utc::now();
            Ok(entry.1.clone())
        }

        async fn complete_billing_event(
            &self, id: Uuid, actual_date: chrono::NaiveDate, completion_pct: f64,
        ) -> AtlasResult<BillingEvent> {
            let mut state = self.state.lock().unwrap();
            let entry = state.events.get_mut(&id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Event {} not found", id)))?;
            entry.0 = "ready".to_string();
            entry.1.status = "ready".to_string();
            entry.1.actual_date = Some(actual_date);
            entry.1.completion_pct = completion_pct;
            entry.1.updated_at = chrono::Utc::now();
            Ok(entry.1.clone())
        }

        async fn delete_billing_event(&self, org_id: Uuid, event_number: &str) -> AtlasResult<()> {
            let mut state = self.state.lock().unwrap();
            let to_remove: Vec<Uuid> = state.events.iter()
                .filter(|(_, (_, e))| e.organization_id == org_id && e.event_number == event_number)
                .map(|(id, _)| *id)
                .collect();
            if to_remove.is_empty() {
                return Err(AtlasError::EntityNotFound(format!("Event '{}' not found", event_number)));
            }
            for id in to_remove {
                state.events.remove(&id);
            }
            Ok(())
        }

        // Invoices
        async fn create_invoice(
            &self, org_id: Uuid, invoice_number: &str, project_id: Uuid,
            project_number: Option<&str>, project_name: Option<&str>,
            invoice_type: &str, customer_id: Option<Uuid>, customer_name: Option<&str>,
            invoice_amount: f64, tax_amount: f64, retention_held: f64,
            total_amount: f64, currency_code: &str,
            billing_period_start: Option<chrono::NaiveDate>,
            billing_period_end: Option<chrono::NaiveDate>,
            invoice_date: chrono::NaiveDate, due_date: Option<chrono::NaiveDate>,
            billing_event_id: Option<Uuid>,
            customer_po_number: Option<&str>, contract_number: Option<&str>,
            notes: Option<&str>, created_by: Option<Uuid>,
        ) -> AtlasResult<ProjectInvoiceHeader> {
            let id = self.next_id();
            let invoice = ProjectInvoiceHeader {
                id,
                organization_id: org_id,
                invoice_number: invoice_number.to_string(),
                project_id,
                project_number: project_number.unwrap_or("").to_string(),
                project_name: project_name.unwrap_or("").to_string(),
                invoice_type: invoice_type.to_string(),
                status: "draft".to_string(),
                customer_id,
                customer_name: customer_name.unwrap_or("").to_string(),
                invoice_amount,
                tax_amount,
                retention_held,
                total_amount,
                currency_code: currency_code.to_string(),
                exchange_rate: 1.0,
                billing_period_start,
                billing_period_end,
                invoice_date,
                due_date,
                billing_event_id,
                customer_po_number: customer_po_number.unwrap_or("").to_string(),
                contract_number: contract_number.unwrap_or("").to_string(),
                gl_posted_flag: false,
                gl_posted_date: None,
                approved_by: None,
                approved_at: None,
                rejected_reason: String::new(),
                payment_status: "unpaid".to_string(),
                payment_date: None,
                notes: notes.unwrap_or("").to_string(),
                metadata: serde_json::json!({}),
                created_by,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let mut state = self.state.lock().unwrap();
            state.invoices.insert(id, ("draft".to_string(), invoice.clone()));
            Ok(invoice)
        }

        async fn get_invoice(&self, id: Uuid) -> AtlasResult<Option<ProjectInvoiceHeader>> {
            let state = self.state.lock().unwrap();
            Ok(state.invoices.get(&id).map(|(_, i)| i.clone()))
        }

        async fn get_invoice_by_number(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<ProjectInvoiceHeader>> {
            let state = self.state.lock().unwrap();
            for (_, (_, inv)) in &state.invoices {
                if inv.organization_id == org_id && inv.invoice_number == invoice_number {
                    return Ok(Some(inv.clone()));
                }
            }
            Ok(None)
        }

        async fn list_invoices(
            &self, org_id: Uuid, project_id: Option<Uuid>, status: Option<&str>,
        ) -> AtlasResult<Vec<ProjectInvoiceHeader>> {
            let state = self.state.lock().unwrap();
            let mut result = Vec::new();
            for (_, (st, inv)) in &state.invoices {
                if inv.organization_id == org_id
                    && (project_id.is_none() || inv.project_id == project_id.unwrap())
                    && (status.is_none() || st == status.unwrap())
                {
                    result.push(inv.clone());
                }
            }
            Ok(result)
        }

        async fn update_invoice_status(&self, id: Uuid, status: &str) -> AtlasResult<ProjectInvoiceHeader> {
            let mut state = self.state.lock().unwrap();
            let entry = state.invoices.get_mut(&id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;
            entry.0 = status.to_string();
            entry.1.status = status.to_string();
            entry.1.updated_at = chrono::Utc::now();
            Ok(entry.1.clone())
        }

        async fn reject_invoice(&self, id: Uuid, reason: &str) -> AtlasResult<ProjectInvoiceHeader> {
            let mut state = self.state.lock().unwrap();
            let entry = state.invoices.get_mut(&id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;
            entry.0 = "rejected".to_string();
            entry.1.status = "rejected".to_string();
            entry.1.rejected_reason = reason.to_string();
            entry.1.updated_at = chrono::Utc::now();
            Ok(entry.1.clone())
        }

        async fn mark_invoice_posted(&self, id: Uuid) -> AtlasResult<ProjectInvoiceHeader> {
            let mut state = self.state.lock().unwrap();
            let entry = state.invoices.get_mut(&id)
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Invoice {} not found", id)))?;
            entry.0 = "posted".to_string();
            entry.1.status = "posted".to_string();
            entry.1.gl_posted_flag = true;
            entry.1.gl_posted_date = Some(chrono::Utc::now());
            entry.1.updated_at = chrono::Utc::now();
            Ok(entry.1.clone())
        }

        async fn delete_invoice(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<()> {
            let mut state = self.state.lock().unwrap();
            let to_remove: Vec<Uuid> = state.invoices.iter()
                .filter(|(_, (_, inv))| inv.organization_id == org_id && inv.invoice_number == invoice_number)
                .map(|(id, _)| *id)
                .collect();
            if to_remove.is_empty() {
                return Err(AtlasError::EntityNotFound(format!("Invoice '{}' not found", invoice_number)));
            }
            for id in to_remove {
                state.invoices.remove(&id);
            }
            Ok(())
        }

        // Invoice Lines
        async fn create_invoice_line(
            &self, org_id: Uuid, invoice_header_id: Uuid, line_number: i32,
            line_source: &str, expenditure_item_id: Option<Uuid>,
            billing_event_id: Option<Uuid>,
            task_id: Option<Uuid>, task_number: Option<&str>, task_name: Option<&str>,
            description: Option<&str>,
            employee_id: Option<Uuid>, employee_name: Option<&str>,
            role_name: Option<&str>, expenditure_type: Option<&str>,
            quantity: f64, unit_of_measure: &str, bill_rate: f64,
            raw_cost_amount: f64, bill_amount: f64, markup_amount: f64,
            retention_amount: f64, tax_amount: f64,
            transaction_date: Option<chrono::NaiveDate>,
        ) -> AtlasResult<ProjectInvoiceLine> {
            let id = self.next_id();
            let line = ProjectInvoiceLine {
                id,
                organization_id: org_id,
                invoice_header_id,
                line_number,
                line_source: line_source.to_string(),
                expenditure_item_id,
                billing_event_id,
                task_id,
                task_number: task_number.unwrap_or("").to_string(),
                task_name: task_name.unwrap_or("").to_string(),
                description: description.unwrap_or("").to_string(),
                employee_id,
                employee_name: employee_name.unwrap_or("").to_string(),
                role_name: role_name.unwrap_or("").to_string(),
                expenditure_type: expenditure_type.unwrap_or("").to_string(),
                quantity,
                unit_of_measure: unit_of_measure.to_string(),
                bill_rate,
                raw_cost_amount,
                bill_amount,
                markup_amount,
                retention_amount,
                tax_amount,
                transaction_date,
                metadata: serde_json::json!({}),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let mut state = self.state.lock().unwrap();
            state.invoice_lines.insert(id, line.clone());
            Ok(line)
        }

        async fn list_invoice_lines(&self, invoice_header_id: Uuid) -> AtlasResult<Vec<ProjectInvoiceLine>> {
            let state = self.state.lock().unwrap();
            let result: Vec<ProjectInvoiceLine> = state.invoice_lines.values()
                .filter(|l| l.invoice_header_id == invoice_header_id)
                .cloned()
                .collect();
            Ok(result)
        }

        // Dashboard
        async fn get_dashboard(&self, _org_id: Uuid) -> AtlasResult<ProjectBillingDashboard> {
            Ok(ProjectBillingDashboard {
                total_projects_billable: 0,
                total_contract_value: 0.0,
                total_billed: 0.0,
                total_unbilled: 0.0,
                total_retention_held: 0.0,
                total_retention_released: 0.0,
                total_invoices: 0,
                draft_invoices: 0,
                submitted_invoices: 0,
                approved_invoices: 0,
                posted_invoices: 0,
                overdue_invoices: 0,
                total_revenue_recognized: 0.0,
                by_billing_method: serde_json::json!({}),
                by_invoice_status: serde_json::json!({}),
                billing_trend: serde_json::json!({}),
            })
        }
    }

    // ========================================================================
    // Schedule Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_schedule_success() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let org_id = Uuid::new_v4();

        let result = engine.create_schedule(
            org_id, "SCH-001", "Standard Rates", Some("Default schedule"),
            "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            None,
            15.0, None,
        ).await;

        assert!(result.is_ok());
        let schedule = result.unwrap();
        assert_eq!(schedule.schedule_number, "SCH-001");
        assert_eq!(schedule.name, "Standard Rates");
        assert_eq!(schedule.status, "draft");
        assert_eq!(schedule.schedule_type, "standard");
    }

    #[tokio::test]
    async fn test_create_schedule_duplicate_number() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        engine.create_schedule(
            org_id, "SCH-DUP", "First", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();

        let result = engine.create_schedule(
            org_id, "SCH-DUP", "Second", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("already exists"));
    }

    #[tokio::test]
    async fn test_create_schedule_invalid_type() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.create_schedule(
            Uuid::new_v4(), "SCH-01", "Bad Type", None,
            "invalid_type", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Invalid schedule_type"));
    }

    #[tokio::test]
    async fn test_create_schedule_end_before_start() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.create_schedule(
            Uuid::new_v4(), "SCH-01", "Bad Dates", None,
            "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            0.0, None,
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("effective_end must be >= effective_start"));
    }

    #[tokio::test]
    async fn test_create_schedule_negative_markup() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.create_schedule(
            Uuid::new_v4(), "SCH-01", "Neg Markup", None,
            "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None,
            -5.0, None,
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("default_markup_pct must be >= 0"));
    }

    #[tokio::test]
    async fn test_activate_schedule() {
        let repo = MockBillingRepo::new();
        let repo_clone = repo.cloned();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        let schedule = engine.create_schedule(
            org_id, "SCH-ACT", "Activate Me", None,
            "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();

        let activated = engine.activate_schedule(schedule.id).await.unwrap();
        assert_eq!(activated.status, "active");
    }

    #[tokio::test]
    async fn test_list_schedules_filter_status() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        let s1 = engine.create_schedule(
            org_id, "SCH-01", "Draft", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();

        let s2 = engine.create_schedule(
            org_id, "SCH-02", "Active", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();
        engine.activate_schedule(s2.id).await.unwrap();

        let drafts = engine.list_schedules(org_id, Some("draft")).await.unwrap();
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].schedule_number, "SCH-01");

        let actives = engine.list_schedules(org_id, Some("active")).await.unwrap();
        assert_eq!(actives.len(), 1);
        assert_eq!(actives[0].schedule_number, "SCH-02");
    }

    #[tokio::test]
    async fn test_delete_schedule() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        engine.create_schedule(
            org_id, "SCH-DEL", "Delete Me", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();

        engine.delete_schedule(org_id, "SCH-DEL").await.unwrap();

        let found = engine.get_schedule_by_number(org_id, "SCH-DEL").await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_delete_schedule_not_found() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.delete_schedule(Uuid::new_v4(), "NONEXISTENT").await;
        assert!(result.is_err());
    }

    // ========================================================================
    // Rate Line Tests
    // ========================================================================

    #[tokio::test]
    async fn test_add_rate_line_success() {
        let repo = MockBillingRepo::new();
        let repo_clone = repo.cloned();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        let schedule = engine.create_schedule(
            org_id, "SCH-RL", "Rate Line Test", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();

        let line = engine.add_rate_line(
            org_id, schedule.id, "Senior Developer", None,
            175.0, "hours",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            None, Some(15.0),
        ).await.unwrap();

        assert_eq!(line.role_name, "Senior Developer");
        assert!((line.bill_rate - 175.0).abs() < 0.01);
        assert_eq!(line.unit_of_measure, "hours");
    }

    #[tokio::test]
    async fn test_add_rate_line_negative_rate() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        let schedule = engine.create_schedule(
            org_id, "SCH-NEG", "Neg Rate", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();

        let result = engine.add_rate_line(
            org_id, schedule.id, "Developer", None,
            -50.0, "hours",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Bill rate must be >= 0"));
    }

    #[tokio::test]
    async fn test_add_rate_line_schedule_not_found() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.add_rate_line(
            Uuid::new_v4(), Uuid::new_v4(), "Developer", None,
            150.0, "hours",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None,
        ).await;

        assert!(result.is_err());
        let err_str = format!("{:?}", result.unwrap_err());
        assert!(err_str.contains("Schedule") && err_str.contains("not found"));
    }

    #[tokio::test]
    async fn test_find_rate_for_role() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        let schedule = engine.create_schedule(
            org_id, "SCH-FIND", "Find Rate", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();

        engine.add_rate_line(
            org_id, schedule.id, "Junior Dev", None,
            100.0, "hours",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None,
        ).await.unwrap();

        engine.add_rate_line(
            org_id, schedule.id, "Senior Dev", None,
            175.0, "hours",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None,
        ).await.unwrap();

        let date = chrono::NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let senior_rate = engine.find_rate_for_role(schedule.id, "Senior Dev", date).await.unwrap();
        assert!(senior_rate.is_some());
        assert!((senior_rate.unwrap().bill_rate - 175.0).abs() < 0.01);

        let unknown = engine.find_rate_for_role(schedule.id, "Architect", date).await.unwrap();
        assert!(unknown.is_none());
    }

    #[tokio::test]
    async fn test_find_rate_respects_effectivity() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        let schedule = engine.create_schedule(
            org_id, "SCH-EFF", "Effectivity Test", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();

        // Rate effective Jan 1 - Mar 31
        engine.add_rate_line(
            org_id, schedule.id, "Dev", None,
            100.0, "hours",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            Some(chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap()),
            None,
        ).await.unwrap();

        // Within range
        let feb = chrono::NaiveDate::from_ymd_opt(2024, 2, 15).unwrap();
        assert!(engine.find_rate_for_role(schedule.id, "Dev", feb).await.unwrap().is_some());

        // Outside range
        let may = chrono::NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
        assert!(engine.find_rate_for_role(schedule.id, "Dev", may).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_and_delete_rate_lines() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        let schedule = engine.create_schedule(
            org_id, "SCH-LD", "List/Delete", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();

        let l1 = engine.add_rate_line(
            org_id, schedule.id, "Role A", None,
            100.0, "hours",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None,
        ).await.unwrap();

        let l2 = engine.add_rate_line(
            org_id, schedule.id, "Role B", None,
            150.0, "hours",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, None,
        ).await.unwrap();

        let lines = engine.list_rate_lines(schedule.id).await.unwrap();
        assert_eq!(lines.len(), 2);

        engine.delete_rate_line(l1.id).await.unwrap();

        let lines_after = engine.list_rate_lines(schedule.id).await.unwrap();
        assert_eq!(lines_after.len(), 1);
        assert_eq!(lines_after[0].role_name, "Role B");
    }

    // ========================================================================
    // Billing Config Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_billing_config_t_and_m() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        // Create schedule first (required for T&M)
        let schedule = engine.create_schedule(
            org_id, "SCH-TM", "T&M Rates", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();

        let project_id = Uuid::new_v4();
        let config = engine.create_billing_config(
            org_id, project_id, "time_and_materials",
            Some(schedule.id), 100000.0, "USD", "detailed", "monthly",
            30, 10.0, 5000.0,
            None, Some("Acme Corp"), Some("PO-123"), Some("CTR-001"), None,
        ).await.unwrap();

        assert_eq!(config.billing_method, "time_and_materials");
        assert_eq!(config.customer_name, "Acme Corp");
        assert_eq!(config.status, "draft");
        assert!((config.retention_pct - 10.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_create_billing_config_fixed_price_no_schedule_needed() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let org_id = Uuid::new_v4();

        let config = engine.create_billing_config(
            org_id, Uuid::new_v4(), "fixed_price",
            None, 500000.0, "USD", "summary", "milestone",
            45, 0.0, 0.0,
            None, Some("Big Corp"), None, None, None,
        ).await.unwrap();

        assert_eq!(config.billing_method, "fixed_price");
    }

    #[tokio::test]
    async fn test_create_billing_config_tm_requires_schedule() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.create_billing_config(
            Uuid::new_v4(), Uuid::new_v4(), "time_and_materials",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("bill_rate_schedule_id is required"));
    }

    #[tokio::test]
    async fn test_create_billing_config_duplicate_project() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let result = engine.create_billing_config(
            org_id, project_id, "milestone",
            None, 200000.0, "USD", "summary", "milestone",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("already exists"));
    }

    #[tokio::test]
    async fn test_create_billing_config_invalid_method() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.create_billing_config(
            Uuid::new_v4(), Uuid::new_v4(), "invalid_method",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Invalid billing_method"));
    }

    #[tokio::test]
    async fn test_create_billing_config_negative_retention() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.create_billing_config(
            Uuid::new_v4(), Uuid::new_v4(), "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, -5.0, 0.0, None, None, None, None, None,
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Retention pct must be 0-100"));
    }

    #[tokio::test]
    async fn test_activate_billing_config() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        let config = engine.create_billing_config(
            org_id, Uuid::new_v4(), "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let active = engine.activate_billing_config(config.id).await.unwrap();
        assert_eq!(active.status, "active");
    }

    #[tokio::test]
    async fn test_cancel_billing_config() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        let config = engine.create_billing_config(
            org_id, Uuid::new_v4(), "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let cancelled = engine.cancel_billing_config(config.id).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    // ========================================================================
    // Billing Event Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_billing_event_success() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        let event = engine.create_billing_event(
            org_id, project_id, "EVT-001", "Phase 1 Complete",
            Some("First milestone delivery"), "milestone",
            25000.0, "USD", 0.0,
            Some(chrono::NaiveDate::from_ymd_opt(2024, 3, 31).unwrap()),
            None, None, None,
        ).await.unwrap();

        assert_eq!(event.event_number, "EVT-001");
        assert_eq!(event.event_type, "milestone");
        assert_eq!(event.status, "planned");
        assert!((event.billing_amount - 25000.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_create_billing_event_invalid_type() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.create_billing_event(
            Uuid::new_v4(), Uuid::new_v4(), "EVT-BAD", "Bad Type", None,
            "invalid_type", 10000.0, "USD", 0.0, None, None, None, None,
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Invalid event_type"));
    }

    #[tokio::test]
    async fn test_create_billing_event_invalid_completion() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.create_billing_event(
            Uuid::new_v4(), Uuid::new_v4(), "EVT-PCT", "Bad Pct", None,
            "progress", 10000.0, "USD", 150.0, None, None, None, None,
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Completion pct must be 0-100"));
    }

    #[tokio::test]
    async fn test_create_billing_event_negative_amount() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.create_billing_event(
            Uuid::new_v4(), Uuid::new_v4(), "EVT-NEG", "Neg Amount", None,
            "milestone", -5000.0, "USD", 0.0, None, None, None, None,
        ).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Billing amount must be >= 0"));
    }

    #[tokio::test]
    async fn test_create_billing_event_duplicate() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let org_id = Uuid::new_v4();

        engine.create_billing_event(
            org_id, Uuid::new_v4(), "EVT-DUP", "First", None,
            "milestone", 10000.0, "USD", 0.0, None, None, None, None,
        ).await.unwrap();

        let result = engine.create_billing_event(
            org_id, Uuid::new_v4(), "EVT-DUP", "Second", None,
            "milestone", 20000.0, "USD", 0.0, None, None, None, None,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("already exists"));
    }

    #[tokio::test]
    async fn test_complete_billing_event() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        let event = engine.create_billing_event(
            org_id, project_id, "EVT-COMP", "Completable", None,
            "milestone", 25000.0, "USD", 0.0, None, None, None, None,
        ).await.unwrap();

        let completed = engine.complete_billing_event(
            event.id,
            chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            100.0,
        ).await.unwrap();

        assert_eq!(completed.status, "ready");
        assert!((completed.completion_pct - 100.0).abs() < 0.01);
        assert!(completed.actual_date.is_some());
    }

    #[tokio::test]
    async fn test_complete_billing_event_wrong_status() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        let event = engine.create_billing_event(
            org_id, Uuid::new_v4(), "EVT-WS", "Wrong Status", None,
            "milestone", 25000.0, "USD", 0.0, None, None, None, None,
        ).await.unwrap();

        // Complete it first
        engine.complete_billing_event(
            event.id,
            chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
            100.0,
        ).await.unwrap();

        // Try to complete again (now in "ready" status)
        let result = engine.complete_billing_event(
            event.id,
            chrono::NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
            100.0,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Must be 'planned'"));
    }

    #[tokio::test]
    async fn test_cancel_billing_event() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        let event = engine.create_billing_event(
            org_id, Uuid::new_v4(), "EVT-CAN", "Cancellable", None,
            "milestone", 10000.0, "USD", 0.0, None, None, None, None,
        ).await.unwrap();

        let cancelled = engine.cancel_billing_event(event.id).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_list_billing_events_by_project() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let proj1 = Uuid::new_v4();
        let proj2 = Uuid::new_v4();

        engine.create_billing_event(
            org_id, proj1, "EVT-P1-1", "P1 Event 1", None,
            "milestone", 10000.0, "USD", 0.0, None, None, None, None,
        ).await.unwrap();

        engine.create_billing_event(
            org_id, proj1, "EVT-P1-2", "P1 Event 2", None,
            "progress", 5000.0, "USD", 0.0, None, None, None, None,
        ).await.unwrap();

        engine.create_billing_event(
            org_id, proj2, "EVT-P2-1", "P2 Event 1", None,
            "milestone", 20000.0, "USD", 0.0, None, None, None, None,
        ).await.unwrap();

        let proj1_events = engine.list_billing_events(org_id, Some(proj1), None).await.unwrap();
        assert_eq!(proj1_events.len(), 2);

        let proj2_events = engine.list_billing_events(org_id, Some(proj2), None).await.unwrap();
        assert_eq!(proj2_events.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_billing_event() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();

        engine.create_billing_event(
            org_id, Uuid::new_v4(), "EVT-DEL", "Deletable", None,
            "milestone", 10000.0, "USD", 0.0, None, None, None, None,
        ).await.unwrap();

        engine.delete_billing_event(org_id, "EVT-DEL").await.unwrap();
        assert!(engine.get_billing_event_by_number(org_id, "EVT-DEL").await.unwrap().is_none());
    }

    // ========================================================================
    // Invoice Tests
    // ========================================================================

    #[tokio::test]
    async fn test_create_invoice_t_and_m() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        // Set up billing config with 10% retention
        let schedule = engine.create_schedule(
            org_id, "SCH-TM-INV", "T&M Rates", None, "standard", "USD",
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None, 0.0, None,
        ).await.unwrap();

        let _config = engine.create_billing_config(
            org_id, project_id, "time_and_materials",
            Some(schedule.id), 100000.0, "USD", "detailed", "monthly",
            30, 10.0, 0.0, None, Some("Test Customer"), None, None, None,
        ).await.unwrap();

        let invoice = engine.create_invoice(
            org_id, "INV-001", project_id,
            Some("PRJ-001"), Some("Test Project"),
            "t_and_m", None, Some("Test Customer"), None,
            vec![
                InvoiceLineRequest {
                    line_source: "expenditure_item".to_string(),
                    expenditure_item_id: None,
                    billing_event_id: None,
                    task_id: None,
                    task_number: Some("TASK-01".to_string()),
                    task_name: Some("Development".to_string()),
                    description: Some("Senior Developer - 40 hrs".to_string()),
                    employee_id: None,
                    employee_name: Some("John Doe".to_string()),
                    role_name: Some("Senior Dev".to_string()),
                    expenditure_type: Some("labor".to_string()),
                    quantity: 40.0,
                    unit_of_measure: "hours".to_string(),
                    bill_rate: 175.0,
                    raw_cost_amount: 5000.0,
                    bill_amount: 7000.0,
                    markup_amount: 0.0,
                    tax_amount: 0.0,
                    transaction_date: Some(chrono::NaiveDate::from_ymd_opt(2024, 2, 15).unwrap()),
                },
            ],
            None, None, None, None,
        ).await.unwrap();

        assert_eq!(invoice.invoice_number, "INV-001");
        assert_eq!(invoice.status, "draft");
        assert_eq!(invoice.invoice_type, "t_and_m");
        // bill_amount = 7000, retention = 700 (10%), total = 7000 - 700 = 6300
        assert!((invoice.invoice_amount - 7000.0).abs() < 0.01);
        assert!((invoice.retention_held - 700.0).abs() < 0.01);
        assert!((invoice.total_amount - 6300.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_create_invoice_milestone() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        // Fixed price, no retention
        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "summary", "milestone",
            30, 0.0, 0.0, None, Some("Big Corp"), None, None, None,
        ).await.unwrap();

        let event = engine.create_billing_event(
            org_id, project_id, "EVT-INV", "Phase 1", None,
            "milestone", 25000.0, "USD", 0.0, None, None, None, None,
        ).await.unwrap();

        let invoice = engine.create_invoice(
            org_id, "INV-M001", project_id,
            Some("PRJ-FP"), Some("Fixed Price Project"),
            "milestone", None, Some("Big Corp"), Some(event.id),
            vec![
                InvoiceLineRequest {
                    line_source: "billing_event".to_string(),
                    expenditure_item_id: None,
                    billing_event_id: Some(event.id),
                    task_id: None,
                    task_number: None,
                    task_name: None,
                    description: Some("Phase 1 milestone".to_string()),
                    employee_id: None,
                    employee_name: None,
                    role_name: None,
                    expenditure_type: None,
                    quantity: 1.0,
                    unit_of_measure: "each".to_string(),
                    bill_rate: 25000.0,
                    raw_cost_amount: 0.0,
                    bill_amount: 25000.0,
                    markup_amount: 0.0,
                    tax_amount: 0.0,
                    transaction_date: Some(chrono::Utc::now().date_naive()),
                },
            ],
            None, None, None, None,
        ).await.unwrap();

        assert_eq!(invoice.invoice_type, "milestone");
        assert!((invoice.invoice_amount - 25000.0).abs() < 0.01);
        assert!((invoice.retention_held - 0.0).abs() < 0.01); // no retention
        assert!((invoice.total_amount - 25000.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_create_invoice_no_lines_fails() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let result = engine.create_invoice(
            Uuid::new_v4(), "INV-NOL", Uuid::new_v4(),
            None, None, "t_and_m", None, None, None,
            vec![], // empty lines
            None, None, None, None,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("at least one line"));
    }

    #[tokio::test]
    async fn test_create_invoice_duplicate_number() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let line = InvoiceLineRequest {
            line_source: "manual".to_string(),
            expenditure_item_id: None, billing_event_id: None,
            task_id: None, task_number: None, task_name: None,
            description: None, employee_id: None, employee_name: None,
            role_name: None, expenditure_type: None,
            quantity: 1.0, unit_of_measure: "each".to_string(),
            bill_rate: 1000.0, raw_cost_amount: 0.0, bill_amount: 1000.0,
            markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
        };

        engine.create_invoice(
            org_id, "INV-DUP", project_id, None, None, "t_and_m", None, None, None,
            vec![line.clone()], None, None, None, None,
        ).await.unwrap();

        let result = engine.create_invoice(
            org_id, "INV-DUP", project_id, None, None, "t_and_m", None, None, None,
            vec![line], None, None, None, None,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("already exists"));
    }

    #[tokio::test]
    async fn test_create_invoice_negative_bill_amount() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());

        let result = engine.create_invoice(
            Uuid::new_v4(), "INV-NEG", Uuid::new_v4(),
            None, None, "t_and_m", None, None, None,
            vec![InvoiceLineRequest {
                line_source: "manual".to_string(),
                expenditure_item_id: None, billing_event_id: None,
                task_id: None, task_number: None, task_name: None,
                description: None, employee_id: None, employee_name: None,
                role_name: None, expenditure_type: None,
                quantity: 1.0, unit_of_measure: "each".to_string(),
                bill_rate: 100.0, raw_cost_amount: 0.0, bill_amount: -500.0,
                markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
            }],
            None, None, None, None,
        ).await;

        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Line bill amount must be >= 0"));
    }

    #[tokio::test]
    async fn test_invoice_lifecycle_submit_approve_post() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let invoice = engine.create_invoice(
            org_id, "INV-LC", project_id, None, None, "t_and_m", None, None, None,
            vec![InvoiceLineRequest {
                line_source: "manual".to_string(),
                expenditure_item_id: None, billing_event_id: None,
                task_id: None, task_number: None, task_name: None,
                description: None, employee_id: None, employee_name: None,
                role_name: None, expenditure_type: None,
                quantity: 1.0, unit_of_measure: "each".to_string(),
                bill_rate: 1000.0, raw_cost_amount: 0.0, bill_amount: 1000.0,
                markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
            }],
            None, None, None, None,
        ).await.unwrap();

        // Submit
        let submitted = engine.submit_invoice(invoice.id).await.unwrap();
        assert_eq!(submitted.status, "submitted");

        // Approve
        let approved = engine.approve_invoice(invoice.id).await.unwrap();
        assert_eq!(approved.status, "approved");

        // Post
        let posted = engine.post_invoice(invoice.id).await.unwrap();
        assert_eq!(posted.status, "posted");
        assert!(posted.gl_posted_flag);
    }

    #[tokio::test]
    async fn test_submit_invoice_wrong_status() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let invoice = engine.create_invoice(
            org_id, "INV-SUB", project_id, None, None, "t_and_m", None, None, None,
            vec![InvoiceLineRequest {
                line_source: "manual".to_string(),
                expenditure_item_id: None, billing_event_id: None,
                task_id: None, task_number: None, task_name: None,
                description: None, employee_id: None, employee_name: None,
                role_name: None, expenditure_type: None,
                quantity: 1.0, unit_of_measure: "each".to_string(),
                bill_rate: 1000.0, raw_cost_amount: 0.0, bill_amount: 1000.0,
                markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
            }],
            None, None, None, None,
        ).await.unwrap();

        // Submit
        engine.submit_invoice(invoice.id).await.unwrap();

        // Try to submit again
        let result = engine.submit_invoice(invoice.id).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Must be 'draft'"));
    }

    #[tokio::test]
    async fn test_reject_invoice() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let invoice = engine.create_invoice(
            org_id, "INV-REJ", project_id, None, None, "t_and_m", None, None, None,
            vec![InvoiceLineRequest {
                line_source: "manual".to_string(),
                expenditure_item_id: None, billing_event_id: None,
                task_id: None, task_number: None, task_name: None,
                description: None, employee_id: None, employee_name: None,
                role_name: None, expenditure_type: None,
                quantity: 1.0, unit_of_measure: "each".to_string(),
                bill_rate: 1000.0, raw_cost_amount: 0.0, bill_amount: 1000.0,
                markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
            }],
            None, None, None, None,
        ).await.unwrap();

        engine.submit_invoice(invoice.id).await.unwrap();

        let rejected = engine.reject_invoice(invoice.id, "Missing backup documentation").await.unwrap();
        assert_eq!(rejected.status, "rejected");
        assert_eq!(rejected.rejected_reason, "Missing backup documentation");
    }

    #[tokio::test]
    async fn test_reject_invoice_not_submitted() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let invoice = engine.create_invoice(
            org_id, "INV-REJ2", project_id, None, None, "t_and_m", None, None, None,
            vec![InvoiceLineRequest {
                line_source: "manual".to_string(),
                expenditure_item_id: None, billing_event_id: None,
                task_id: None, task_number: None, task_name: None,
                description: None, employee_id: None, employee_name: None,
                role_name: None, expenditure_type: None,
                quantity: 1.0, unit_of_measure: "each".to_string(),
                bill_rate: 1000.0, raw_cost_amount: 0.0, bill_amount: 1000.0,
                markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
            }],
            None, None, None, None,
        ).await.unwrap();

        // Try to reject a draft
        let result = engine.reject_invoice(invoice.id, "Wrong").await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Must be 'submitted'"));
    }

    #[tokio::test]
    async fn test_cancel_invoice() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let invoice = engine.create_invoice(
            org_id, "INV-CAN", project_id, None, None, "t_and_m", None, None, None,
            vec![InvoiceLineRequest {
                line_source: "manual".to_string(),
                expenditure_item_id: None, billing_event_id: None,
                task_id: None, task_number: None, task_name: None,
                description: None, employee_id: None, employee_name: None,
                role_name: None, expenditure_type: None,
                quantity: 1.0, unit_of_measure: "each".to_string(),
                bill_rate: 1000.0, raw_cost_amount: 0.0, bill_amount: 1000.0,
                markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
            }],
            None, None, None, None,
        ).await.unwrap();

        let cancelled = engine.cancel_invoice(invoice.id).await.unwrap();
        assert_eq!(cancelled.status, "cancelled");
    }

    #[tokio::test]
    async fn test_cancel_posted_invoice_fails() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let invoice = engine.create_invoice(
            org_id, "INV-CPT", project_id, None, None, "t_and_m", None, None, None,
            vec![InvoiceLineRequest {
                line_source: "manual".to_string(),
                expenditure_item_id: None, billing_event_id: None,
                task_id: None, task_number: None, task_name: None,
                description: None, employee_id: None, employee_name: None,
                role_name: None, expenditure_type: None,
                quantity: 1.0, unit_of_measure: "each".to_string(),
                bill_rate: 1000.0, raw_cost_amount: 0.0, bill_amount: 1000.0,
                markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
            }],
            None, None, None, None,
        ).await.unwrap();

        engine.submit_invoice(invoice.id).await.unwrap();
        engine.approve_invoice(invoice.id).await.unwrap();
        engine.post_invoice(invoice.id).await.unwrap();

        let result = engine.cancel_invoice(invoice.id).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Cannot cancel a posted invoice"));
    }

    #[tokio::test]
    async fn test_approve_before_submit_fails() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let invoice = engine.create_invoice(
            org_id, "INV-APR", project_id, None, None, "t_and_m", None, None, None,
            vec![InvoiceLineRequest {
                line_source: "manual".to_string(),
                expenditure_item_id: None, billing_event_id: None,
                task_id: None, task_number: None, task_name: None,
                description: None, employee_id: None, employee_name: None,
                role_name: None, expenditure_type: None,
                quantity: 1.0, unit_of_measure: "each".to_string(),
                bill_rate: 1000.0, raw_cost_amount: 0.0, bill_amount: 1000.0,
                markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
            }],
            None, None, None, None,
        ).await.unwrap();

        let result = engine.approve_invoice(invoice.id).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Must be 'submitted'"));
    }

    #[tokio::test]
    async fn test_post_before_approve_fails() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let invoice = engine.create_invoice(
            org_id, "INV-POST", project_id, None, None, "t_and_m", None, None, None,
            vec![InvoiceLineRequest {
                line_source: "manual".to_string(),
                expenditure_item_id: None, billing_event_id: None,
                task_id: None, task_number: None, task_name: None,
                description: None, employee_id: None, employee_name: None,
                role_name: None, expenditure_type: None,
                quantity: 1.0, unit_of_measure: "each".to_string(),
                bill_rate: 1000.0, raw_cost_amount: 0.0, bill_amount: 1000.0,
                markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
            }],
            None, None, None, None,
        ).await.unwrap();

        let result = engine.post_invoice(invoice.id).await;
        assert!(result.is_err());
        assert!(format!("{:?}", result.unwrap_err()).contains("Must be 'approved'"));
    }

    #[tokio::test]
    async fn test_list_invoices_by_project() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let proj1 = Uuid::new_v4();
        let proj2 = Uuid::new_v4();

        engine.create_billing_config(
            org_id, proj1, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        engine.create_billing_config(
            org_id, proj2, "fixed_price",
            None, 200000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let line = InvoiceLineRequest {
            line_source: "manual".to_string(),
            expenditure_item_id: None, billing_event_id: None,
            task_id: None, task_number: None, task_name: None,
            description: None, employee_id: None, employee_name: None,
            role_name: None, expenditure_type: None,
            quantity: 1.0, unit_of_measure: "each".to_string(),
            bill_rate: 1000.0, raw_cost_amount: 0.0, bill_amount: 1000.0,
            markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
        };

        engine.create_invoice(
            org_id, "INV-P1-1", proj1, None, None, "t_and_m", None, None, None,
            vec![line.clone()], None, None, None, None,
        ).await.unwrap();

        engine.create_invoice(
            org_id, "INV-P1-2", proj1, None, None, "t_and_m", None, None, None,
            vec![line.clone()], None, None, None, None,
        ).await.unwrap();

        engine.create_invoice(
            org_id, "INV-P2-1", proj2, None, None, "t_and_m", None, None, None,
            vec![line], None, None, None, None,
        ).await.unwrap();

        let proj1_invoices = engine.list_invoices(org_id, Some(proj1), None).await.unwrap();
        assert_eq!(proj1_invoices.len(), 2);

        let proj2_invoices = engine.list_invoices(org_id, Some(proj2), None).await.unwrap();
        assert_eq!(proj2_invoices.len(), 1);
    }

    #[tokio::test]
    async fn test_get_invoice_lines() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        let invoice = engine.create_invoice(
            org_id, "INV-LN", project_id, None, None, "t_and_m", None, None, None,
            vec![
                InvoiceLineRequest {
                    line_source: "expenditure_item".to_string(),
                    expenditure_item_id: None, billing_event_id: None,
                    task_id: None, task_number: Some("T1".to_string()),
                    task_name: Some("Task 1".to_string()),
                    description: Some("Line 1".to_string()),
                    employee_id: None, employee_name: Some("Alice".to_string()),
                    role_name: Some("Senior Dev".to_string()),
                    expenditure_type: Some("labor".to_string()),
                    quantity: 40.0, unit_of_measure: "hours".to_string(),
                    bill_rate: 175.0, raw_cost_amount: 5000.0,
                    bill_amount: 7000.0, markup_amount: 0.0,
                    tax_amount: 0.0, transaction_date: None,
                },
                InvoiceLineRequest {
                    line_source: "expenditure_item".to_string(),
                    expenditure_item_id: None, billing_event_id: None,
                    task_id: None, task_number: Some("T2".to_string()),
                    task_name: Some("Task 2".to_string()),
                    description: Some("Line 2".to_string()),
                    employee_id: None, employee_name: Some("Bob".to_string()),
                    role_name: Some("Architect".to_string()),
                    expenditure_type: Some("labor".to_string()),
                    quantity: 20.0, unit_of_measure: "hours".to_string(),
                    bill_rate: 225.0, raw_cost_amount: 3500.0,
                    bill_amount: 4500.0, markup_amount: 0.0,
                    tax_amount: 0.0, transaction_date: None,
                },
            ],
            None, None, None, None,
        ).await.unwrap();

        let lines = engine.get_invoice_lines(invoice.id).await.unwrap();
        assert_eq!(lines.len(), 2);
        let names: std::collections::HashSet<&str> = lines.iter().map(|l| l.employee_name.as_str()).collect();
        assert!(names.contains("Alice"));
        assert!(names.contains("Bob"));
    }

    #[tokio::test]
    async fn test_delete_invoice() {
        let repo = MockBillingRepo::new();
        let engine = ProjectBillingEngine::new(repo.into_repo());
        let org_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();

        engine.create_billing_config(
            org_id, project_id, "fixed_price",
            None, 100000.0, "USD", "detailed", "monthly",
            30, 0.0, 0.0, None, None, None, None, None,
        ).await.unwrap();

        engine.create_invoice(
            org_id, "INV-DEL", project_id, None, None, "t_and_m", None, None, None,
            vec![InvoiceLineRequest {
                line_source: "manual".to_string(),
                expenditure_item_id: None, billing_event_id: None,
                task_id: None, task_number: None, task_name: None,
                description: None, employee_id: None, employee_name: None,
                role_name: None, expenditure_type: None,
                quantity: 1.0, unit_of_measure: "each".to_string(),
                bill_rate: 1000.0, raw_cost_amount: 0.0, bill_amount: 1000.0,
                markup_amount: 0.0, tax_amount: 0.0, transaction_date: None,
            }],
            None, None, None, None,
        ).await.unwrap();

        engine.delete_invoice(org_id, "INV-DEL").await.unwrap();
        assert!(engine.get_invoice_by_number(org_id, "INV-DEL").await.unwrap().is_none());
    }

    // ========================================================================
    // Retention Computation Tests
    // ========================================================================

    #[test]
    fn test_compute_retention_basic() {
        let ret = compute_retention(10000.0, 10.0, 0.0);
        assert!((ret - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_retention_with_cap() {
        let ret = compute_retention(100000.0, 10.0, 5000.0);
        assert!((ret - 5000.0).abs() < 0.01); // 10% would be 10000, but capped at 5000
    }

    #[test]
    fn test_compute_retention_zero_pct() {
        let ret = compute_retention(10000.0, 0.0, 0.0);
        assert!((ret - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_retention_cap_not_reached() {
        let ret = compute_retention(1000.0, 10.0, 5000.0);
        assert!((ret - 100.0).abs() < 0.01); // 10% is 100, well under cap
    }

    // ========================================================================
    // Dashboard Test
    // ========================================================================

    #[tokio::test]
    async fn test_get_dashboard() {
        let engine = ProjectBillingEngine::new(MockBillingRepo::new().into_repo());
        let dashboard = engine.get_dashboard(Uuid::new_v4()).await.unwrap();
        assert_eq!(dashboard.total_projects_billable, 0);
        assert_eq!(dashboard.total_invoices, 0);
    }

    // ========================================================================
    // Validation Constant Tests
    // ========================================================================

    #[test]
    fn test_valid_schedule_types() {
        assert!(VALID_SCHEDULE_TYPES.contains(&"standard"));
        assert!(VALID_SCHEDULE_TYPES.contains(&"overtime"));
        assert!(VALID_SCHEDULE_TYPES.contains(&"holiday"));
        assert!(VALID_SCHEDULE_TYPES.contains(&"custom"));
        assert!(!VALID_SCHEDULE_TYPES.contains(&"invalid"));
    }

    #[test]
    fn test_valid_billing_methods() {
        assert!(VALID_BILLING_METHODS.contains(&"time_and_materials"));
        assert!(VALID_BILLING_METHODS.contains(&"fixed_price"));
        assert!(VALID_BILLING_METHODS.contains(&"milestone"));
        assert!(VALID_BILLING_METHODS.contains(&"cost_plus"));
        assert!(VALID_BILLING_METHODS.contains(&"retention"));
    }

    #[test]
    fn test_valid_event_types() {
        assert!(VALID_EVENT_TYPES.contains(&"milestone"));
        assert!(VALID_EVENT_TYPES.contains(&"progress"));
        assert!(VALID_EVENT_TYPES.contains(&"completion"));
        assert!(VALID_EVENT_TYPES.contains(&"retention_release"));
    }

    #[test]
    fn test_valid_invoice_statuses() {
        assert!(VALID_INVOICE_STATUSES.contains(&"draft"));
        assert!(VALID_INVOICE_STATUSES.contains(&"submitted"));
        assert!(VALID_INVOICE_STATUSES.contains(&"approved"));
        assert!(VALID_INVOICE_STATUSES.contains(&"rejected"));
        assert!(VALID_INVOICE_STATUSES.contains(&"posted"));
        assert!(VALID_INVOICE_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_valid_invoice_types() {
        assert!(VALID_INVOICE_TYPES.contains(&"progress"));
        assert!(VALID_INVOICE_TYPES.contains(&"milestone"));
        assert!(VALID_INVOICE_TYPES.contains(&"t_and_m"));
        assert!(VALID_INVOICE_TYPES.contains(&"retention_release"));
        assert!(VALID_INVOICE_TYPES.contains(&"debit_memo"));
        assert!(VALID_INVOICE_TYPES.contains(&"credit_memo"));
    }
}
