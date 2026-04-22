//! AutoInvoice Engine Implementation
//!
//! Oracle Fusion Cloud Receivables AutoInvoice.
//! Automatically creates AR invoices from imported transaction data
//! with configurable validation rules, grouping rules, and line ordering.
//!
//! The process follows Oracle Fusion's AutoInvoice pipeline:
//! 1. Import lines into interface tables
//! 2. Validate each line against validation rules
//! 3. Group valid lines into invoices using grouping rules
//! 4. Create invoice headers and lines
//! 5. Post completed invoices

use atlas_shared::{
    AutoInvoiceBatch, AutoInvoiceLine, AutoInvoiceGroupingRule,
    AutoInvoiceValidationRule, AutoInvoiceResult, AutoInvoiceResultLine,
    AutoInvoiceImportRequest,
    AutoInvoiceValidationError,
    AtlasError, AtlasResult,
};
use super::AutoInvoiceRepository;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid transaction types for AutoInvoice
#[allow(dead_code)]
const VALID_TRANSACTION_TYPES: &[&str] = &[
    "invoice", "credit_memo", "debit_memo", "on_account_credit",
];

/// Valid validation types
#[allow(dead_code)]
const VALID_VALIDATION_TYPES: &[&str] = &[
    "required", "format", "reference", "range", "custom",
];

/// Valid batch statuses
#[allow(dead_code)]
const VALID_BATCH_STATUSES: &[&str] = &[
    "pending", "validating", "validated", "processing", "completed", "failed", "cancelled",
];

/// Required fields for any AutoInvoice line
const REQUIRED_LINE_FIELDS: &[&str] = &[
    "transaction_type", "currency_code", "transaction_date", "gl_date",
];

/// AutoInvoice engine for processing AR invoice creation
pub struct AutoInvoiceEngine {
    repository: Arc<dyn AutoInvoiceRepository>,
}

impl AutoInvoiceEngine {
    pub fn new(repository: Arc<dyn AutoInvoiceRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Grouping Rule Management
    // ========================================================================

    /// Create a new grouping rule
    pub async fn create_grouping_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        transaction_types: serde_json::Value,
        group_by_fields: serde_json::Value,
        line_order_by: serde_json::Value,
        is_default: bool,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceGroupingRule> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Grouping rule name is required".to_string()));
        }

        // Validate group_by_fields is an array of strings
        if !group_by_fields.is_array() {
            return Err(AtlasError::ValidationFailed("group_by_fields must be a JSON array".to_string()));
        }

        // Check uniqueness
        if self.repository.get_grouping_rule_by_name(org_id, name).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Grouping rule '{}' already exists", name
            )));
        }

        // If setting as default, unset any existing default
        if is_default {
            if let Some(existing) = self.repository.get_default_grouping_rule(org_id).await? {
                return Err(AtlasError::Conflict(format!(
                    "A default grouping rule already exists: '{}'. Unset it first.", existing.name
                )));
            }
        }

        info!("Creating AutoInvoice grouping rule '{}' for org {}", name, org_id);

        self.repository.create_grouping_rule(
            org_id, name, description, transaction_types,
            group_by_fields, line_order_by, is_default, priority, created_by,
        ).await
    }

    /// Get a grouping rule by ID
    pub async fn get_grouping_rule(&self, id: Uuid) -> AtlasResult<Option<AutoInvoiceGroupingRule>> {
        self.repository.get_grouping_rule(id).await
    }

    /// List all grouping rules
    pub async fn list_grouping_rules(&self, org_id: Uuid) -> AtlasResult<Vec<AutoInvoiceGroupingRule>> {
        self.repository.list_grouping_rules(org_id).await
    }

    /// Delete a grouping rule
    pub async fn delete_grouping_rule(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting AutoInvoice grouping rule {}", id);
        self.repository.delete_grouping_rule(id).await
    }

    // ========================================================================
    // Validation Rule Management
    // ========================================================================

    /// Create a new validation rule
    pub async fn create_validation_rule(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        field_name: &str,
        validation_type: &str,
        validation_expression: Option<&str>,
        error_message: &str,
        is_fatal: bool,
        transaction_types: serde_json::Value,
        priority: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceValidationRule> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Validation rule name is required".to_string()));
        }
        if field_name.is_empty() {
            return Err(AtlasError::ValidationFailed("Field name is required".to_string()));
        }
        if !VALID_VALIDATION_TYPES.contains(&validation_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid validation_type '{}'. Must be one of: {}",
                validation_type, VALID_VALIDATION_TYPES.join(", ")
            )));
        }
        if error_message.is_empty() {
            return Err(AtlasError::ValidationFailed("Error message is required".to_string()));
        }

        info!("Creating AutoInvoice validation rule '{}' for org {}", name, org_id);

        self.repository.create_validation_rule(
            org_id, name, description, field_name, validation_type,
            validation_expression, error_message, is_fatal,
            transaction_types, priority, effective_from, effective_to,
            created_by,
        ).await
    }

    /// List all validation rules
    pub async fn list_validation_rules(&self, org_id: Uuid) -> AtlasResult<Vec<AutoInvoiceValidationRule>> {
        self.repository.list_validation_rules(org_id).await
    }

    /// Delete a validation rule
    pub async fn delete_validation_rule(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleting AutoInvoice validation rule {}", id);
        self.repository.delete_validation_rule(id).await
    }

    // ========================================================================
    // Batch Import
    // ========================================================================

    /// Import transaction lines as a new AutoInvoice batch
    /// This creates a batch and inserts all lines in pending status
    pub async fn import_batch(
        &self,
        org_id: Uuid,
        request: &AutoInvoiceImportRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceBatch> {
        if request.lines.is_empty() {
            return Err(AtlasError::ValidationFailed("Cannot import an empty batch".to_string()));
        }

        // Resolve grouping rule
        let grouping_rule_id = if let Some(rule_id) = request.grouping_rule_id {
            let rule = self.repository.get_grouping_rule(rule_id).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Grouping rule {} not found", rule_id)
                ))?;
            Some(rule.id)
        } else {
            self.repository.get_default_grouping_rule(org_id).await?.map(|r| r.id)
        };

        // Generate batch number
        let batch_number = format!("AI-{}", chrono::Utc::now().format("%Y%m%d%H%M%S%f"));

        info!("Importing AutoInvoice batch {} with {} lines", batch_number, request.lines.len());

        // Create batch
        let batch = self.repository.create_batch(
            org_id,
            &batch_number,
            &request.batch_source,
            request.description.as_deref(),
            grouping_rule_id,
            created_by,
        ).await?;

        // Insert lines
        let today = chrono::Utc::now().date_naive();
        for (idx, line_req) in request.lines.iter().enumerate() {
            let line_number = (idx + 1) as i32;
            let transaction_type = line_req.transaction_type.as_deref().unwrap_or("invoice");
            let transaction_date = line_req.transaction_date.unwrap_or(today);
            let gl_date = line_req.gl_date.unwrap_or(today);
            let unit_price = line_req.unit_price.as_deref().unwrap_or("0");
            let line_amount = line_req.line_amount.as_deref().unwrap_or("0");

            self.repository.create_line(
                org_id,
                batch.id,
                line_number,
                line_req.source_line_id.as_deref(),
                transaction_type,
                line_req.customer_id,
                line_req.customer_number.as_deref(),
                line_req.customer_name.as_deref(),
                line_req.bill_to_customer_id,
                line_req.bill_to_site_id,
                line_req.ship_to_customer_id,
                line_req.ship_to_site_id,
                line_req.item_code.as_deref(),
                line_req.item_description.as_deref(),
                line_req.quantity.as_deref(),
                line_req.unit_of_measure.as_deref(),
                unit_price,
                line_amount,
                &line_req.currency_code,
                line_req.exchange_rate.as_deref(),
                transaction_date,
                gl_date,
                line_req.due_date,
                line_req.revenue_account_code.as_deref(),
                line_req.receivable_account_code.as_deref(),
                line_req.tax_code.as_deref(),
                line_req.tax_amount.as_deref(),
                line_req.sales_rep_id,
                line_req.sales_rep_name.as_deref(),
                line_req.memo_line.as_deref(),
                line_req.reference_number.as_deref(),
                line_req.sales_order_number.as_deref(),
                line_req.sales_order_line.as_deref(),
                created_by,
            ).await?;
        }

        // Update batch total count
        let lines = self.repository.list_lines_by_batch(batch.id).await?;
        self.repository.update_batch_counts(
            batch.id,
            lines.len() as i32,
            0,
            0,
            0,
            "0",
            serde_json::json!([]),
        ).await?;

        // Reload batch
        self.repository.get_batch(batch.id).await?
            .ok_or_else(|| AtlasError::EntityNotFound("Created batch not found".to_string()))
    }

    /// Get a batch by ID
    pub async fn get_batch(&self, id: Uuid) -> AtlasResult<Option<AutoInvoiceBatch>> {
        self.repository.get_batch(id).await
    }

    /// List batches
    pub async fn list_batches(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<AutoInvoiceBatch>> {
        self.repository.list_batches(org_id, status).await
    }

    // ========================================================================
    // Validation
    // ========================================================================

    /// Validate all lines in a batch against configured validation rules
    /// Updates each line's status to 'valid' or 'invalid'
    pub async fn validate_batch(&self, batch_id: Uuid) -> AtlasResult<AutoInvoiceBatch> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Batch {} not found", batch_id)
            ))?;

        if batch.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot validate batch in '{}' status. Must be 'pending'.", batch.status
            )));
        }

        // Update status to validating
        let mut batch = self.repository.update_batch_status(batch_id, "validating").await?;

        // Load all lines
        let lines = self.repository.list_lines_by_batch(batch_id).await?;

        // Load validation rules for this org
        let all_rules = self.repository.get_validation_rules(batch.organization_id, None).await?;

        let mut valid_count = 0i32;
        let mut invalid_count = 0i32;
        let mut all_errors: Vec<AutoInvoiceValidationError> = Vec::new();

        // Validate each line
        for line in &lines {
            let mut line_errors: Vec<AutoInvoiceValidationError> = Vec::new();

            // Apply built-in required field validation
            for field in REQUIRED_LINE_FIELDS {
                let value = self.get_line_field_value(line, field);
                if value.is_empty() {
                    line_errors.push(AutoInvoiceValidationError {
                        line_number: line.line_number,
                        field_name: field.to_string(),
                        validation_rule: "built_in_required".to_string(),
                        error_message: format!("Field '{}' is required", field),
                        is_fatal: true,
                    });
                }
            }

            // Apply built-in transaction type validation
            if !VALID_TRANSACTION_TYPES.contains(&line.transaction_type.as_str()) {
                line_errors.push(AutoInvoiceValidationError {
                    line_number: line.line_number,
                    field_name: "transaction_type".to_string(),
                    validation_rule: "built_in_transaction_type".to_string(),
                    error_message: format!(
                        "Invalid transaction_type '{}'. Must be one of: {}",
                        line.transaction_type, VALID_TRANSACTION_TYPES.join(", ")
                    ),
                    is_fatal: true,
                });
            }

            // Apply built-in amount validation
            let line_amount: f64 = line.line_amount.parse().unwrap_or(0.0);
            if line_amount < 0.0 {
                line_errors.push(AutoInvoiceValidationError {
                    line_number: line.line_number,
                    field_name: "line_amount".to_string(),
                    validation_rule: "built_in_non_negative".to_string(),
                    error_message: "Line amount cannot be negative".to_string(),
                    is_fatal: true,
                });
            }

            // Apply custom validation rules
            for rule in &all_rules {
                if !rule.is_active {
                    continue;
                }

                // Check if rule applies to this transaction type
                let applies = rule.transaction_types.as_array()
                    .map(|arr| arr.iter().any(|v| v.as_str() == Some(&line.transaction_type)))
                    .unwrap_or(true);

                if !applies {
                    continue;
                }

                // Check date effectiveness
                if let Some(from) = rule.effective_from {
                    if line.transaction_date < from {
                        continue;
                    }
                }
                if let Some(to) = rule.effective_to {
                    if line.transaction_date > to {
                        continue;
                    }
                }

                let value = self.get_line_field_value(line, &rule.field_name);
                let error = self.apply_validation_rule(rule, &value, line.line_number);

                if let Some(err) = error {
                    line_errors.push(err);
                }
            }

            // Update line status
            let has_fatal = line_errors.iter().any(|e| e.is_fatal);
            if has_fatal {
                self.repository.update_line_status(
                    line.id,
                    "invalid",
                    serde_json::to_value(&line_errors).unwrap_or_default(),
                ).await?;
                invalid_count += 1;
            } else {
                self.repository.update_line_status(
                    line.id,
                    "valid",
                    serde_json::to_value(&line_errors).unwrap_or_default(),
                ).await?;
                valid_count += 1;
            }

            all_errors.extend(line_errors);
        }

        // Update batch counts and status
        self.repository.update_batch_counts(
            batch_id,
            lines.len() as i32,
            valid_count,
            invalid_count,
            0,
            "0",
            serde_json::to_value(&all_errors).unwrap_or_default(),
        ).await?;

        let new_status = if invalid_count > 0 && valid_count == 0 {
            "failed"
        } else {
            "validated"
        };

        batch = self.repository.update_batch_status(batch_id, new_status).await?;
        info!("AutoInvoice batch {} validated: {} valid, {} invalid", batch_id, valid_count, invalid_count);

        Ok(batch)
    }

    // ========================================================================
    // Grouping & Invoice Creation
    // ========================================================================

    /// Process a validated batch: group lines into invoices and create AR invoices
    pub async fn process_batch(&self, batch_id: Uuid) -> AtlasResult<AutoInvoiceBatch> {
        let batch = self.repository.get_batch(batch_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Batch {} not found", batch_id)
            ))?;

        if batch.status != "validated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot process batch in '{}' status. Must be 'validated'.", batch.status
            )));
        }

        // Update status to processing
        let mut batch = self.repository.update_batch_status(batch_id, "processing").await?;

        // Get valid lines
        let valid_lines = self.repository.list_lines_by_status(batch_id, "valid").await?;

        if valid_lines.is_empty() {
            self.repository.update_batch_status(batch_id, "failed").await?;
            return Err(AtlasError::ValidationFailed(
                "No valid lines to process".to_string(),
            ));
        }

        // Resolve grouping rule
        let grouping_rule = if let Some(rule_id) = batch.grouping_rule_id {
            self.repository.get_grouping_rule(rule_id).await?
        } else {
            self.repository.get_default_grouping_rule(batch.organization_id).await?
        };

        // Group lines
        let groups = self.group_lines(&valid_lines, grouping_rule.as_ref());

        // Create invoices for each group
        let mut invoices_created = 0i32;
        let mut total_invoice_amount = 0.0f64;
        let mut invoice_counter = 0i32;

        for (_group_key, group_lines) in &groups {
            invoice_counter += 1;
            let first_line = &group_lines[0];

            // Generate invoice number
            let invoice_number = format!("INV-{}-{:04}", batch.batch_number, invoice_counter);

            // Determine common fields from first line
            let due_date = first_line.due_date.or_else(|| {
                Some(first_line.transaction_date + chrono::Duration::days(30))
            });

            let result = self.repository.create_result(
                batch.organization_id,
                batch_id,
                &invoice_number,
                &first_line.transaction_type,
                first_line.customer_id,
                first_line.bill_to_customer_id,
                first_line.bill_to_site_id,
                first_line.ship_to_customer_id,
                first_line.ship_to_site_id,
                &first_line.currency_code,
                first_line.exchange_rate.as_deref(),
                first_line.transaction_date,
                first_line.gl_date,
                due_date,
                first_line.receivable_account_code.as_deref(),
                first_line.sales_rep_id,
                first_line.sales_order_number.as_deref(),
                first_line.reference_number.as_deref(),
                batch.created_by,
            ).await?;

            // Create result lines and link source lines
            let mut subtotal = 0.0f64;
            let mut tax_total = 0.0f64;

            for (idx, line) in group_lines.iter().enumerate() {
                let line_num = (idx + 1) as i32;

                self.repository.create_result_line(
                    batch.organization_id,
                    result.id,
                    line_num,
                    line.source_line_id.as_deref(),
                    line.item_code.as_deref(),
                    line.item_description.as_deref(),
                    line.quantity.as_deref(),
                    line.unit_of_measure.as_deref(),
                    &line.unit_price,
                    &line.line_amount,
                    line.tax_code.as_deref(),
                    line.tax_amount.as_deref(),
                    line.revenue_account_code.as_deref(),
                    line.sales_order_number.as_deref(),
                    line.sales_order_line.as_deref(),
                ).await?;

                let amt: f64 = line.line_amount.parse().unwrap_or(0.0);
                let tax: f64 = line.tax_amount.as_deref()
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);
                subtotal += amt;
                tax_total += tax;

                // Link source line to result
                self.repository.update_line_invoice(line.id, result.id, line_num).await?;
            }

            let total = subtotal + tax_total;
            total_invoice_amount += total;

            // Update result totals
            self.repository.update_result_totals(
                result.id,
                &format!("{:.2}", subtotal),
                &format!("{:.2}", tax_total),
                &format!("{:.2}", total),
                group_lines.len() as i32,
            ).await?;

            invoices_created += 1;
        }

        // Update batch with final counts
        self.repository.update_batch_counts(
            batch_id,
            batch.total_lines,
            batch.valid_lines,
            batch.invalid_lines,
            invoices_created,
            &format!("{:.2}", total_invoice_amount),
            batch.validation_errors.clone(),
        ).await?;

        batch = self.repository.update_batch_status(batch_id, "completed").await?;

        info!(
            "AutoInvoice batch {} processed: {} invoices created, total amount {:.2}",
            batch_id, invoices_created, total_invoice_amount
        );

        Ok(batch)
    }

    /// Convenience method: import, validate, and process in one call
    pub async fn import_and_process(
        &self,
        org_id: Uuid,
        request: &AutoInvoiceImportRequest,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AutoInvoiceBatch> {
        let batch = self.import_batch(org_id, request, created_by).await?;
        let batch = self.validate_batch(batch.id).await?;

        if batch.status == "validated" {
            self.process_batch(batch.id).await
        } else {
            Ok(batch)
        }
    }

    // ========================================================================
    // Query Methods
    // ========================================================================

    /// Get lines for a batch
    pub async fn get_batch_lines(&self, batch_id: Uuid) -> AtlasResult<Vec<AutoInvoiceLine>> {
        self.repository.list_lines_by_batch(batch_id).await
    }

    /// Get invoices created from a batch
    pub async fn get_batch_results(&self, batch_id: Uuid) -> AtlasResult<Vec<AutoInvoiceResult>> {
        self.repository.list_results_by_batch(batch_id).await
    }

    /// Get lines for an invoice
    pub async fn get_invoice_lines(&self, invoice_id: Uuid) -> AtlasResult<Vec<AutoInvoiceResultLine>> {
        self.repository.list_result_lines(invoice_id).await
    }

    /// Get an invoice by invoice number
    pub async fn get_invoice_by_number(&self, org_id: Uuid, invoice_number: &str) -> AtlasResult<Option<AutoInvoiceResult>> {
        self.repository.get_result_by_invoice_number(org_id, invoice_number).await
    }

    /// Update invoice status (e.g., to 'posted')
    pub async fn update_invoice_status(&self, invoice_id: Uuid, status: &str) -> AtlasResult<AutoInvoiceResult> {
        let valid_statuses = ["draft", "complete", "posted", "cancelled"];
        if !valid_statuses.contains(&status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid invoice status '{}'. Must be one of: {}", status, valid_statuses.join(", ")
            )));
        }

        let invoice = self.repository.get_result(invoice_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Invoice {} not found", invoice_id)
            ))?;

        // Status transition validation
        match (invoice.status.as_str(), status) {
            ("draft", "complete") | ("draft", "cancelled") |
            ("complete", "posted") | ("complete", "cancelled") => {},
            _ => {
                return Err(AtlasError::WorkflowError(format!(
                    "Cannot transition invoice from '{}' to '{}'", invoice.status, status
                )));
            }
        }

        info!("Updating AutoInvoice {} status to {}", invoice.invoice_number, status);
        self.repository.update_result_status(invoice_id, status).await
    }

    /// Get AutoInvoice summary for dashboard
    pub async fn get_summary(&self, org_id: Uuid) -> AtlasResult<atlas_shared::AutoInvoiceSummary> {
        self.repository.get_summary(org_id).await
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Get the string value of a field from a line by field name
    fn get_line_field_value(&self, line: &AutoInvoiceLine, field: &str) -> String {
        match field {
            "transaction_type" => line.transaction_type.clone(),
            "customer_id" => line.customer_id.map(|id| id.to_string()).unwrap_or_default(),
            "customer_number" => line.customer_number.clone().unwrap_or_default(),
            "customer_name" => line.customer_name.clone().unwrap_or_default(),
            "bill_to_customer_id" => line.bill_to_customer_id.map(|id| id.to_string()).unwrap_or_default(),
            "bill_to_site_id" => line.bill_to_site_id.map(|id| id.to_string()).unwrap_or_default(),
            "currency_code" => line.currency_code.clone(),
            "transaction_date" => line.transaction_date.to_string(),
            "gl_date" => line.gl_date.to_string(),
            "line_amount" => line.line_amount.clone(),
            "unit_price" => line.unit_price.clone(),
            "item_code" => line.item_code.clone().unwrap_or_default(),
            "item_description" => line.item_description.clone().unwrap_or_default(),
            "revenue_account_code" => line.revenue_account_code.clone().unwrap_or_default(),
            "receivable_account_code" => line.receivable_account_code.clone().unwrap_or_default(),
            "tax_code" => line.tax_code.clone().unwrap_or_default(),
            "sales_order_number" => line.sales_order_number.clone().unwrap_or_default(),
            "reference_number" => line.reference_number.clone().unwrap_or_default(),
            _ => String::new(),
        }
    }

    /// Apply a validation rule to a field value
    fn apply_validation_rule(
        &self,
        rule: &AutoInvoiceValidationRule,
        value: &str,
        line_number: i32,
    ) -> Option<AutoInvoiceValidationError> {
        let failed = match rule.validation_type.as_str() {
            "required" => value.is_empty(),
            "format" => {
                if let Some(expr) = &rule.validation_expression {
                    // Simple pattern matching (could use regex for complex patterns)
                    match expr.as_str() {
                        "numeric" => value.parse::<f64>().is_err() && !value.is_empty(),
                        "uppercase" => value != value.to_uppercase() && !value.is_empty(),
                        "email" => !value.contains('@') && !value.is_empty(),
                        _ => false,
                    }
                } else {
                    false
                }
            }
            "reference" => {
                // In a real system, this would check if the reference exists
                // For now, just check non-empty
                value.is_empty()
            }
            "range" => {
                if let Some(expr) = &rule.validation_expression {
                    if let Ok(val) = value.parse::<f64>() {
                        // Parse range expression "min:max"
                        let parts: Vec<&str> = expr.split(':').collect();
                        if parts.len() == 2 {
                            let min: f64 = parts[0].parse().unwrap_or(f64::MIN);
                            let max: f64 = parts[1].parse().unwrap_or(f64::MAX);
                            val < min || val > max
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            "custom" => false, // Custom validations would be handled by external handlers
            _ => false,
        };

        if failed {
            Some(AutoInvoiceValidationError {
                line_number,
                field_name: rule.field_name.clone(),
                validation_rule: rule.name.clone(),
                error_message: rule.error_message.clone(),
                is_fatal: rule.is_fatal,
            })
        } else {
            None
        }
    }

    /// Group lines into invoice groups based on grouping rule fields
    fn group_lines<'a>(
        &self,
        lines: &'a [AutoInvoiceLine],
        rule: Option<&AutoInvoiceGroupingRule>,
    ) -> Vec<(String, Vec<&'a AutoInvoiceLine>)> {
        // Default grouping fields if no rule specified
        let group_fields: Vec<String> = rule
            .and_then(|r| r.group_by_fields.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(|| {
                vec![
                    "bill_to_customer_id".to_string(),
                    "currency_code".to_string(),
                    "transaction_type".to_string(),
                ]
            });

        // Group by the specified fields
        let mut groups: HashMap<String, Vec<&AutoInvoiceLine>> = HashMap::new();

        for line in lines {
            let key: String = group_fields.iter()
                .map(|f| {
                    let val = self.get_line_field_value(line, f);
                    if val.is_empty() { "__empty__".to_string() } else { val }
                })
                .collect::<Vec<_>>()
                .join("|");

            groups.entry(key).or_default().push(line);
        }

        // Sort groups by key for deterministic ordering
        let mut sorted_groups: Vec<(String, Vec<&AutoInvoiceLine>)> = groups.into_iter().collect();
        sorted_groups.sort_by(|a, b| a.0.cmp(&b.0));

        sorted_groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Unit tests for validation logic
    // ========================================================================

    #[test]
    fn test_valid_transaction_types() {
        assert!(VALID_TRANSACTION_TYPES.contains(&"invoice"));
        assert!(VALID_TRANSACTION_TYPES.contains(&"credit_memo"));
        assert!(VALID_TRANSACTION_TYPES.contains(&"debit_memo"));
        assert!(VALID_TRANSACTION_TYPES.contains(&"on_account_credit"));
        assert!(!VALID_TRANSACTION_TYPES.contains(&"invalid"));
        assert!(!VALID_TRANSACTION_TYPES.contains(&"purchase_order"));
    }

    #[test]
    fn test_valid_validation_types() {
        assert!(VALID_VALIDATION_TYPES.contains(&"required"));
        assert!(VALID_VALIDATION_TYPES.contains(&"format"));
        assert!(VALID_VALIDATION_TYPES.contains(&"reference"));
        assert!(VALID_VALIDATION_TYPES.contains(&"range"));
        assert!(VALID_VALIDATION_TYPES.contains(&"custom"));
        assert!(!VALID_VALIDATION_TYPES.contains(&"unknown"));
    }

    #[test]
    fn test_valid_batch_statuses() {
        assert!(VALID_BATCH_STATUSES.contains(&"pending"));
        assert!(VALID_BATCH_STATUSES.contains(&"validating"));
        assert!(VALID_BATCH_STATUSES.contains(&"validated"));
        assert!(VALID_BATCH_STATUSES.contains(&"processing"));
        assert!(VALID_BATCH_STATUSES.contains(&"completed"));
        assert!(VALID_BATCH_STATUSES.contains(&"failed"));
        assert!(VALID_BATCH_STATUSES.contains(&"cancelled"));
    }

    #[test]
    fn test_required_line_fields() {
        assert!(REQUIRED_LINE_FIELDS.contains(&"transaction_type"));
        assert!(REQUIRED_LINE_FIELDS.contains(&"currency_code"));
        assert!(REQUIRED_LINE_FIELDS.contains(&"transaction_date"));
        assert!(REQUIRED_LINE_FIELDS.contains(&"gl_date"));
        assert!(!REQUIRED_LINE_FIELDS.contains(&"item_code"));
    }

    #[test]
    fn test_line_field_value_extraction() {
        let repo = Arc::new(crate::mock_repos::MockAutoInvoiceRepository);
        let engine = AutoInvoiceEngine::new(repo);

        let line = AutoInvoiceLine {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            batch_id: Uuid::new_v4(),
            line_number: 1,
            source_line_id: Some("SRC-001".to_string()),
            transaction_type: "invoice".to_string(),
            customer_id: Some(Uuid::new_v4()),
            customer_number: Some("CUST001".to_string()),
            customer_name: Some("Test Customer".to_string()),
            bill_to_customer_id: Some(Uuid::new_v4()),
            bill_to_site_id: None,
            ship_to_customer_id: None,
            ship_to_site_id: None,
            item_code: Some("ITEM001".to_string()),
            item_description: Some("Test Item".to_string()),
            quantity: Some("10".to_string()),
            unit_of_measure: Some("EA".to_string()),
            unit_price: "100.00".to_string(),
            line_amount: "1000.00".to_string(),
            currency_code: "USD".to_string(),
            exchange_rate: None,
            transaction_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            gl_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            due_date: None,
            revenue_account_code: Some("4000".to_string()),
            receivable_account_code: Some("1200".to_string()),
            tax_code: Some("VAT20".to_string()),
            tax_amount: Some("200.00".to_string()),
            sales_rep_id: None,
            sales_rep_name: None,
            memo_line: None,
            reference_number: Some("REF-001".to_string()),
            sales_order_number: Some("SO-001".to_string()),
            sales_order_line: Some("1".to_string()),
            status: "pending".to_string(),
            validation_errors: serde_json::json!([]),
            invoice_id: None,
            invoice_line_number: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Test field extraction
        assert_eq!(engine.get_line_field_value(&line, "transaction_type"), "invoice");
        assert_eq!(engine.get_line_field_value(&line, "currency_code"), "USD");
        assert_eq!(engine.get_line_field_value(&line, "item_code"), "ITEM001");
        assert_eq!(engine.get_line_field_value(&line, "line_amount"), "1000.00");
        assert_eq!(engine.get_line_field_value(&line, "customer_number"), "CUST001");
        assert_eq!(engine.get_line_field_value(&line, "nonexistent_field"), "");
    }

    #[test]
    fn test_apply_validation_rule_required() {
        let repo = Arc::new(crate::mock_repos::MockAutoInvoiceRepository);
        let engine = AutoInvoiceEngine::new(repo);

        let rule = AutoInvoiceValidationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "customer_required".to_string(),
            description: None,
            field_name: "customer_number".to_string(),
            validation_type: "required".to_string(),
            validation_expression: None,
            error_message: "Customer number is required".to_string(),
            is_fatal: true,
            transaction_types: serde_json::json!(["invoice"]),
            is_active: true,
            priority: 10,
            effective_from: None,
            effective_to: None,
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Empty value should fail required validation
        let result = engine.apply_validation_rule(&rule, "", 1);
        assert!(result.is_some());
        let err = result.unwrap();
        assert_eq!(err.field_name, "customer_number");
        assert!(err.is_fatal);

        // Non-empty value should pass
        let result = engine.apply_validation_rule(&rule, "CUST001", 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_validation_rule_format_numeric() {
        let repo = Arc::new(crate::mock_repos::MockAutoInvoiceRepository);
        let engine = AutoInvoiceEngine::new(repo);

        let rule = AutoInvoiceValidationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "amount_numeric".to_string(),
            description: None,
            field_name: "line_amount".to_string(),
            validation_type: "format".to_string(),
            validation_expression: Some("numeric".to_string()),
            error_message: "Amount must be numeric".to_string(),
            is_fatal: true,
            transaction_types: serde_json::json!(["invoice"]),
            is_active: true,
            priority: 10,
            effective_from: None,
            effective_to: None,
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Non-numeric value should fail
        let result = engine.apply_validation_rule(&rule, "abc", 1);
        assert!(result.is_some());

        // Numeric value should pass
        let result = engine.apply_validation_rule(&rule, "100.50", 1);
        assert!(result.is_none());

        // Empty value should pass format validation (required handles that)
        let result = engine.apply_validation_rule(&rule, "", 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_validation_rule_format_uppercase() {
        let repo = Arc::new(crate::mock_repos::MockAutoInvoiceRepository);
        let engine = AutoInvoiceEngine::new(repo);

        let rule = AutoInvoiceValidationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "code_uppercase".to_string(),
            description: None,
            field_name: "item_code".to_string(),
            validation_type: "format".to_string(),
            validation_expression: Some("uppercase".to_string()),
            error_message: "Item code must be uppercase".to_string(),
            is_fatal: false,
            transaction_types: serde_json::json!(["invoice"]),
            is_active: true,
            priority: 10,
            effective_from: None,
            effective_to: None,
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Lowercase should fail
        let result = engine.apply_validation_rule(&rule, "item001", 1);
        assert!(result.is_some());
        let err = result.unwrap();
        assert!(!err.is_fatal); // warning, not fatal

        // Uppercase should pass
        let result = engine.apply_validation_rule(&rule, "ITEM001", 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_validation_rule_range() {
        let repo = Arc::new(crate::mock_repos::MockAutoInvoiceRepository);
        let engine = AutoInvoiceEngine::new(repo);

        let rule = AutoInvoiceValidationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "amount_range".to_string(),
            description: None,
            field_name: "line_amount".to_string(),
            validation_type: "range".to_string(),
            validation_expression: Some("0:1000000".to_string()),
            error_message: "Amount must be between 0 and 1,000,000".to_string(),
            is_fatal: true,
            transaction_types: serde_json::json!(["invoice"]),
            is_active: true,
            priority: 10,
            effective_from: None,
            effective_to: None,
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Within range should pass
        let result = engine.apply_validation_rule(&rule, "500", 1);
        assert!(result.is_none());

        // At boundary should pass
        let result = engine.apply_validation_rule(&rule, "0", 1);
        assert!(result.is_none());
        let result = engine.apply_validation_rule(&rule, "1000000", 1);
        assert!(result.is_none());

        // Below range should fail
        let result = engine.apply_validation_rule(&rule, "-1", 1);
        assert!(result.is_some());

        // Above range should fail
        let result = engine.apply_validation_rule(&rule, "1000001", 1);
        assert!(result.is_some());
    }

    #[test]
    fn test_apply_validation_rule_format_email() {
        let repo = Arc::new(crate::mock_repos::MockAutoInvoiceRepository);
        let engine = AutoInvoiceEngine::new(repo);

        let rule = AutoInvoiceValidationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "email_format".to_string(),
            description: None,
            field_name: "customer_name".to_string(),
            validation_type: "format".to_string(),
            validation_expression: Some("email".to_string()),
            error_message: "Must be a valid email".to_string(),
            is_fatal: false,
            transaction_types: serde_json::json!(["invoice"]),
            is_active: true,
            priority: 10,
            effective_from: None,
            effective_to: None,
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // No @ sign should fail
        let result = engine.apply_validation_rule(&rule, "notanemail", 1);
        assert!(result.is_some());

        // Has @ sign should pass (simplified validation)
        let result = engine.apply_validation_rule(&rule, "user@example.com", 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_group_lines_default_fields() {
        let repo = Arc::new(crate::mock_repos::MockAutoInvoiceRepository);
        let engine = AutoInvoiceEngine::new(repo);

        let cust1 = Uuid::new_v4();
        let cust2 = Uuid::new_v4();

        let line1 = make_test_line(1, cust1, "USD", "invoice");
        let line2 = make_test_line(2, cust1, "USD", "invoice"); // same group as line1
        let line3 = make_test_line(3, cust2, "USD", "invoice"); // different customer
        let line4 = make_test_line(4, cust1, "EUR", "invoice"); // different currency
        let line5 = make_test_line(5, cust1, "USD", "credit_memo"); // different type

        let lines = vec![line1, line2, line3, line4, line5];

        // Default grouping: bill_to_customer_id, currency_code, transaction_type
        let groups = engine.group_lines(&lines, None);

        // Should have 4 groups:
        // - cust1 + USD + invoice (2 lines)
        // - cust2 + USD + invoice (1 line)
        // - cust1 + EUR + invoice (1 line)
        // - cust1 + USD + credit_memo (1 line)
        assert_eq!(groups.len(), 4);

        // Find the cust1+USD+invoice group (should have 2 lines)
        let main_group = groups.iter()
            .find(|(key, _lines)| {
                key.contains(&cust1.to_string()) &&
                key.contains("USD") &&
                key.contains("invoice") &&
                !key.contains("credit_memo")
            })
            .unwrap();
        assert_eq!(main_group.1.len(), 2);
    }

    #[test]
    fn test_group_lines_with_custom_rule() {
        let repo = Arc::new(crate::mock_repos::MockAutoInvoiceRepository);
        let engine = AutoInvoiceEngine::new(repo);

        let cust1 = Uuid::new_v4();

        let line1 = make_test_line(1, cust1, "USD", "invoice");
        let line2 = make_test_line(2, cust1, "EUR", "invoice"); // different currency

        let lines = vec![line1, line2];

        // Custom grouping by customer only (no currency) — should produce 1 group
        let rule = AutoInvoiceGroupingRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            name: "by_customer".to_string(),
            description: None,
            transaction_types: serde_json::json!(["invoice"]),
            group_by_fields: serde_json::json!(["bill_to_customer_id"]),
            line_order_by: serde_json::json!(["line_number"]),
            is_default: false,
            is_active: true,
            priority: 10,
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let groups = engine.group_lines(&lines, Some(&rule));
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].1.len(), 2);
    }

    #[test]
    fn test_group_lines_empty_bill_to_uses_empty_key() {
        let repo = Arc::new(crate::mock_repos::MockAutoInvoiceRepository);
        let engine = AutoInvoiceEngine::new(repo);

        let mut line1 = make_test_line(1, Uuid::new_v4(), "USD", "invoice");
        line1.bill_to_customer_id = None;
        let mut line2 = make_test_line(2, Uuid::new_v4(), "USD", "invoice");
        line2.bill_to_customer_id = None;

        let lines = vec![line1, line2];

        // Both have no bill_to_customer_id — should group together
        let groups = engine.group_lines(&lines, None);
        // With default grouping (bill_to_customer_id, currency_code, transaction_type),
        // both lines have empty bill_to, USD currency, invoice type → 1 group
        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn test_invoice_number_format() {
        let batch_number = "AI-20240115123000000000";
        let invoice_number = format!("INV-{}-{:04}", batch_number, 1);
        assert!(invoice_number.starts_with("INV-AI-"));
        assert!(invoice_number.ends_with("-0001"));
    }

    #[test]
    fn test_amount_calculations() {
        let line_amounts = vec!["100.00", "250.50", "75.25"];
        let tax_amounts = vec!["20.00", "50.10", "15.05"];

        let subtotal: f64 = line_amounts.iter()
            .map(|v| v.parse::<f64>().unwrap())
            .sum();
        let tax_total: f64 = tax_amounts.iter()
            .map(|v| v.parse::<f64>().unwrap())
            .sum();
        let total = subtotal + tax_total;

        assert!((subtotal - 425.75).abs() < 0.01);
        assert!((tax_total - 85.15).abs() < 0.01);
        assert!((total - 510.90).abs() < 0.01);
    }

    #[test]
    fn test_invoice_status_transitions() {
        // Valid transitions: draft→complete, draft→cancelled,
        // complete→posted, complete→cancelled
        let valid = [
            ("draft", "complete"),
            ("draft", "cancelled"),
            ("complete", "posted"),
            ("complete", "cancelled"),
        ];
        let invalid = [
            ("draft", "posted"),      // must go through complete first
            ("posted", "complete"),    // can't go back
            ("cancelled", "draft"),    // can't reopen
            ("posted", "cancelled"),   // can't cancel posted
        ];

        for (from, to) in valid {
            let is_valid = matches!((from, to),
                ("draft", "complete") | ("draft", "cancelled") |
                ("complete", "posted") | ("complete", "cancelled")
            );
            assert!(is_valid, "Expected ({}, {}) to be valid", from, to);
        }

        for (from, to) in invalid {
            let is_valid = matches!((from, to),
                ("draft", "complete") | ("draft", "cancelled") |
                ("complete", "posted") | ("complete", "cancelled")
            );
            assert!(!is_valid, "Expected ({}, {}) to be invalid", from, to);
        }
    }

    // ========================================================================
    // Helper functions
    // ========================================================================

    fn make_test_line(
        line_number: i32,
        bill_to_customer_id: Uuid,
        currency_code: &str,
        transaction_type: &str,
    ) -> AutoInvoiceLine {
        AutoInvoiceLine {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            batch_id: Uuid::new_v4(),
            line_number,
            source_line_id: None,
            transaction_type: transaction_type.to_string(),
            customer_id: Some(bill_to_customer_id),
            customer_number: Some(format!("CUST-{}", line_number)),
            customer_name: Some(format!("Customer {}", line_number)),
            bill_to_customer_id: Some(bill_to_customer_id),
            bill_to_site_id: None,
            ship_to_customer_id: None,
            ship_to_site_id: None,
            item_code: Some(format!("ITEM-{}", line_number)),
            item_description: Some(format!("Item {}", line_number)),
            quantity: Some("1".to_string()),
            unit_of_measure: Some("EA".to_string()),
            unit_price: "100.00".to_string(),
            line_amount: "100.00".to_string(),
            currency_code: currency_code.to_string(),
            exchange_rate: None,
            transaction_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            gl_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            due_date: None,
            revenue_account_code: Some("4000".to_string()),
            receivable_account_code: Some("1200".to_string()),
            tax_code: Some("VAT20".to_string()),
            tax_amount: Some("20.00".to_string()),
            sales_rep_id: None,
            sales_rep_name: None,
            memo_line: None,
            reference_number: None,
            sales_order_number: None,
            sales_order_line: None,
            status: "valid".to_string(),
            validation_errors: serde_json::json!([]),
            invoice_id: None,
            invoice_line_number: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}
