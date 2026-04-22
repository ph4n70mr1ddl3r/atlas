//! Subledger Accounting Engine
//!
//! Manages accounting methods, derivation rules, journal entry generation,
//! posting, reversal, transfer to GL, and dashboard reporting.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > General Ledger > Subledger Accounting

use atlas_shared::{
    AccountingMethod, AccountingDerivationRule,
    SubledgerJournalEntry, SubledgerJournalLine,
    SlaEvent, GlTransferLog, SlaDashboardSummary,
    AtlasError, AtlasResult,
};
use super::SubledgerAccountingRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid applications for accounting methods
#[allow(dead_code)]
const VALID_APPLICATIONS: &[&str] = &[
    "payables", "receivables", "expenses", "assets", "projects", "general",
];

/// Valid event classes
#[allow(dead_code)]
const VALID_EVENT_CLASSES: &[&str] = &[
    "create", "update", "cancel", "reverse",
];

/// Valid entry statuses
#[allow(dead_code)]
const VALID_ENTRY_STATUSES: &[&str] = &[
    "draft", "accounted", "posted", "transferred", "reversed", "error",
];

/// Valid line types
#[allow(dead_code)]
const VALID_LINE_TYPES: &[&str] = &[
    "debit", "credit", "tax", "discount", "rounding",
];

/// Valid derivation types
#[allow(dead_code)]
const VALID_DERIVATION_TYPES: &[&str] = &[
    "constant", "lookup", "formula",
];

/// Valid transfer statuses
#[allow(dead_code)]
const VALID_TRANSFER_STATUSES: &[&str] = &[
    "pending", "in_progress", "completed", "failed", "reversed",
];

/// Valid GL transfer statuses
#[allow(dead_code)]
const VALID_GL_TRANSFER_STATUSES: &[&str] = &[
    "pending", "transferred", "failed",
];

/// Subledger Accounting Engine
pub struct SubledgerAccountingEngine {
    repository: Arc<dyn SubledgerAccountingRepository>,
}

impl SubledgerAccountingEngine {
    pub fn new(repository: Arc<dyn SubledgerAccountingRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Accounting Methods
    // ========================================================================

    /// Create or define an accounting method
    pub async fn create_accounting_method(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        application: &str,
        transaction_type: &str,
        event_class: Option<&str>,
        auto_accounting: Option<bool>,
        allow_manual_entries: Option<bool>,
        apply_rounding: Option<bool>,
        rounding_account_code: Option<&str>,
        rounding_threshold: Option<&str>,
        require_balancing: Option<bool>,
        intercompany_balancing_account: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingMethod> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Accounting method code and name are required".to_string(),
            ));
        }
        if !VALID_APPLICATIONS.contains(&application) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid application '{}'. Must be one of: {}",
                application, VALID_APPLICATIONS.join(", ")
            )));
        }
        let ec = event_class.unwrap_or("create");
        if !VALID_EVENT_CLASSES.contains(&ec) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid event_class '{}'. Must be one of: {}",
                ec, VALID_EVENT_CLASSES.join(", ")
            )));
        }

        info!("Creating accounting method {} for {}::{}", code, application, transaction_type);

        self.repository.create_accounting_method(
            org_id, code, name, description,
            application, transaction_type, ec,
            auto_accounting.unwrap_or(true),
            allow_manual_entries.unwrap_or(false),
            apply_rounding.unwrap_or(true),
            rounding_account_code,
            rounding_threshold.unwrap_or("0.01"),
            require_balancing.unwrap_or(true),
            intercompany_balancing_account,
            effective_from, effective_to,
            created_by,
        ).await
    }

    /// Get an accounting method by code
    pub async fn get_accounting_method(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountingMethod>> {
        self.repository.get_accounting_method(org_id, code).await
    }

    /// List accounting methods, optionally filtered by application
    pub async fn list_accounting_methods(&self, org_id: Uuid, application: Option<&str>) -> AtlasResult<Vec<AccountingMethod>> {
        self.repository.list_accounting_methods(org_id, application).await
    }

    /// Delete (soft-delete) an accounting method
    pub async fn delete_accounting_method(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.get_accounting_method(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Accounting method '{}' not found", code)
            ))?;

        info!("Deleting accounting method {} in org {}", code, org_id);
        self.repository.delete_accounting_method(org_id, code).await
    }

    // ========================================================================
    // Accounting Derivation Rules
    // ========================================================================

    /// Create a derivation rule for account code determination
    pub async fn create_derivation_rule(
        &self,
        org_id: Uuid,
        accounting_method_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        line_type: &str,
        priority: i32,
        conditions: serde_json::Value,
        source_field: Option<&str>,
        derivation_type: &str,
        fixed_account_code: Option<&str>,
        account_derivation_lookup: serde_json::Value,
        formula_expression: Option<&str>,
        sequence: i32,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountingDerivationRule> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Derivation rule code and name are required".to_string(),
            ));
        }
        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line_type '{}'. Must be one of: {}",
                line_type, VALID_LINE_TYPES.join(", ")
            )));
        }
        if !VALID_DERIVATION_TYPES.contains(&derivation_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid derivation_type '{}'. Must be one of: {}",
                derivation_type, VALID_DERIVATION_TYPES.join(", ")
            )));
        }

        // Validate derivation type has required fields
        match derivation_type {
            "constant" if fixed_account_code.is_none() => {
                return Err(AtlasError::ValidationFailed(
                    "Constant derivation type requires fixed_account_code".to_string(),
                ));
            }
            "lookup" if source_field.is_none() => {
                return Err(AtlasError::ValidationFailed(
                    "Lookup derivation type requires source_field".to_string(),
                ));
            }
            "formula" if formula_expression.is_none() => {
                return Err(AtlasError::ValidationFailed(
                    "Formula derivation type requires formula_expression".to_string(),
                ));
            }
            _ => {}
        }

        // Verify the accounting method exists
        self.repository.get_accounting_method_by_id(accounting_method_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Accounting method {} not found", accounting_method_id)
            ))?;

        info!("Creating derivation rule {} for method {}", code, accounting_method_id);

        self.repository.create_derivation_rule(
            org_id, accounting_method_id, code, name, description,
            line_type, priority, conditions, source_field,
            derivation_type, fixed_account_code,
            account_derivation_lookup, formula_expression,
            sequence, effective_from, effective_to, created_by,
        ).await
    }

    /// List derivation rules for an accounting method
    pub async fn list_derivation_rules(&self, org_id: Uuid, method_id: Uuid) -> AtlasResult<Vec<AccountingDerivationRule>> {
        self.repository.list_derivation_rules(org_id, method_id).await
    }

    /// List active derivation rules for a specific line type
    pub async fn list_active_derivation_rules(&self, org_id: Uuid, method_id: Uuid, line_type: &str) -> AtlasResult<Vec<AccountingDerivationRule>> {
        self.repository.list_active_derivation_rules(org_id, method_id, line_type).await
    }

    /// Resolve an account code using derivation rules
    ///
    /// Evaluates rules by priority (lowest first) and returns the first match.
    /// For 'constant' type, returns fixed_account_code directly.
    /// For 'lookup' type, looks up the source field value in the lookup map.
    /// For 'formula' type, returns the formula expression (execution is deferred).
    pub fn resolve_account_code(
        &self,
        rules: &[AccountingDerivationRule],
        line_type: &str,
        transaction_attributes: &serde_json::Value,
    ) -> Option<String> {
        // Filter rules by line type and sort by priority
        let mut matching: Vec<&AccountingDerivationRule> = rules.iter()
            .filter(|r| r.line_type == line_type && r.is_active)
            .collect();
        matching.sort_by_key(|r| r.priority);

        for rule in matching {
            // Check conditions
            if !rule.conditions.is_null() && !rule.conditions.as_object().is_none_or(|obj| obj.is_empty())
                && !Self::evaluate_conditions(&rule.conditions, transaction_attributes) {
                    continue;
                }

            return match rule.derivation_type.as_str() {
                "constant" => rule.fixed_account_code.clone(),
                "lookup" => {
                    if let Some(field) = &rule.source_field {
                        if let Some(value) = transaction_attributes.get(field) {
                            let key = if let Some(s) = value.as_str() {
                                s.to_string()
                            } else {
                                value.to_string()
                            };
                            rule.account_derivation_lookup.get(&key)
                                .and_then(|v| v.as_str().map(|s| s.to_string()))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                "formula" => rule.formula_expression.clone(),
                _ => None,
            };
        }
        None
    }

    /// Check if a rule's conditions match the transaction attributes
    fn evaluate_conditions(
        conditions: &serde_json::Value,
        attributes: &serde_json::Value,
    ) -> bool {
        let Some(obj) = conditions.as_object() else { return true; };
        if obj.is_empty() { return true; }

        for (key, expected) in obj {
            if let Some(actual) = attributes.get(key) {
                match expected {
                    serde_json::Value::String(s) => {
                        if actual.as_str().unwrap_or("") != s.as_str() {
                            return false;
                        }
                    }
                    serde_json::Value::Number(n) => {
                        if actual.as_f64().unwrap_or(0.0) != n.as_f64().unwrap_or(0.0) {
                            return false;
                        }
                    }
                    serde_json::Value::Bool(b) => {
                        if actual.as_bool().unwrap_or(false) != *b {
                            return false;
                        }
                    }
                    _ => {}
                }
            } else {
                return false;
            }
        }
        true
    }

    /// Delete a derivation rule
    pub async fn delete_derivation_rule(&self, org_id: Uuid, method_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.delete_derivation_rule(org_id, method_id, code).await
    }

    // ========================================================================
    // Subledger Journal Entries
    // ========================================================================

    /// Create a journal entry from a subledger transaction
    pub async fn create_journal_entry(
        &self,
        org_id: Uuid,
        source_application: &str,
        source_transaction_type: &str,
        source_transaction_id: Uuid,
        source_transaction_number: Option<&str>,
        accounting_method_id: Option<Uuid>,
        description: Option<&str>,
        reference_number: Option<&str>,
        accounting_date: chrono::NaiveDate,
        period_name: Option<&str>,
        currency_code: &str,
        entered_currency_code: &str,
        currency_conversion_date: Option<chrono::NaiveDate>,
        currency_conversion_type: Option<&str>,
        currency_conversion_rate: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<SubledgerJournalEntry> {
        // Generate entry number
        let entry_number = format!("SLA-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        info!("Creating SLA journal entry {} for {}::{}", entry_number, source_application, source_transaction_type);

        self.repository.create_journal_entry(
            org_id, source_application, source_transaction_type,
            source_transaction_id, source_transaction_number,
            accounting_method_id, &entry_number, description,
            reference_number, accounting_date, period_name,
            currency_code, entered_currency_code,
            currency_conversion_date, currency_conversion_type,
            currency_conversion_rate,
            "0.00", "0.00", "0.00", "0.00",  // Will be updated when lines are added
            "draft", None, false,
            created_by,
        ).await
    }

    /// Add a journal line to an entry
    pub async fn add_journal_line(
        &self,
        org_id: Uuid,
        journal_entry_id: Uuid,
        line_type: &str,
        account_code: &str,
        account_description: Option<&str>,
        derivation_rule_id: Option<Uuid>,
        entered_amount: &str,
        accounted_amount: &str,
        currency_code: &str,
        conversion_date: Option<chrono::NaiveDate>,
        conversion_rate: Option<&str>,
        attribute_category: Option<&str>,
        attribute1: Option<&str>,
        attribute2: Option<&str>,
        attribute3: Option<&str>,
        attribute4: Option<&str>,
        attribute5: Option<&str>,
        source_line_id: Option<Uuid>,
        source_line_type: Option<&str>,
        tax_code: Option<&str>,
        tax_rate: Option<&str>,
        tax_amount: Option<&str>,
    ) -> AtlasResult<SubledgerJournalLine> {
        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line_type '{}'. Must be one of: {}",
                line_type, VALID_LINE_TYPES.join(", ")
            )));
        }
        if account_code.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Account code is required".to_string(),
            ));
        }
        let amt: f64 = entered_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Entered amount must be a valid number".to_string(),
        ))?;
        let _accounted: f64 = accounted_amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Accounted amount must be a valid number".to_string(),
        ))?;
        if (line_type == "debit" || line_type == "credit") && amt <= 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Debit and credit amounts must be positive".to_string(),
            ));
        }

        // Get current line count for line_number
        let lines = self.repository.list_journal_lines(journal_entry_id).await?;
        let line_number = (lines.len() + 1) as i32;

        let line = self.repository.create_journal_line(
            org_id, journal_entry_id, line_number, line_type,
            account_code, account_description, derivation_rule_id,
            entered_amount, accounted_amount, currency_code,
            conversion_date, conversion_rate,
            attribute_category, attribute1, attribute2, attribute3, attribute4, attribute5,
            source_line_id, source_line_type,
            tax_code, tax_rate, tax_amount,
        ).await?;

        // Recalculate entry totals and balancing
        self.recalculate_entry_balances(journal_entry_id).await?;

        Ok(line)
    }

    /// Recalculate an entry's totals from its lines and check balancing
    async fn recalculate_entry_balances(&self, journal_entry_id: Uuid) -> AtlasResult<()> {
        let lines = self.repository.list_journal_lines(journal_entry_id).await?;

        let mut total_debit = 0.0_f64;
        let mut total_credit = 0.0_f64;

        for line in &lines {
            let accounted: f64 = line.accounted_amount.parse().unwrap_or(0.0);
            match line.line_type.as_str() {
                "debit" => total_debit += accounted,
                "credit" => total_credit += accounted,
                _ => {} // tax, discount, rounding handled differently
            }
        }

        // Check balancing (debit should equal credit within rounding threshold)
        let is_balanced = (total_debit - total_credit).abs() < 0.01;

        self.repository.update_journal_entry_balances(
            journal_entry_id,
            &format!("{:.2}", total_debit),
            &format!("{:.2}", total_credit),
            &format!("{:.2}", total_debit), // For same-currency, entered = accounted
            &format!("{:.2}", total_credit),
            is_balanced,
        ).await?;

        Ok(())
    }

    /// Get a journal entry by ID
    pub async fn get_journal_entry(&self, id: Uuid) -> AtlasResult<Option<SubledgerJournalEntry>> {
        self.repository.get_journal_entry(id).await
    }

    /// List journal entries with optional filters
    pub async fn list_journal_entries(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        source_application: Option<&str>,
        source_transaction_type: Option<&str>,
        accounting_date_from: Option<chrono::NaiveDate>,
        accounting_date_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<Vec<SubledgerJournalEntry>> {
        if let Some(s) = status {
            if !VALID_ENTRY_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_ENTRY_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_journal_entries(
            org_id, status, source_application, source_transaction_type,
            accounting_date_from, accounting_date_to,
        ).await
    }

    /// Get all lines for a journal entry
    pub async fn list_journal_lines(&self, journal_entry_id: Uuid) -> AtlasResult<Vec<SubledgerJournalLine>> {
        self.repository.list_journal_lines(journal_entry_id).await
    }

    // ========================================================================
    // Posting & Status Transitions
    // ========================================================================

    /// Account a draft journal entry (validate & compute final amounts)
    /// Transition: draft → accounted
    pub async fn account_entry(&self, entry_id: Uuid, accounted_by: Option<Uuid>) -> AtlasResult<SubledgerJournalEntry> {
        let entry = self.repository.get_journal_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Journal entry {} not found", entry_id)
            ))?;

        if entry.status != "draft" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot account entry in '{}' status. Must be 'draft'.",
                entry.status
            )));
        }

        // Verify the entry is balanced
        if !entry.is_balanced {
            return Err(AtlasError::ValidationFailed(
                "Journal entry is not balanced. Debits must equal credits.".to_string(),
            ));
        }

        info!("Accounting SLA journal entry {}", entry.entry_number);
        self.repository.update_journal_entry_status(
            entry_id, "accounted", None, None, None, accounted_by,
        ).await
    }

    /// Post an accounted journal entry
    /// Transition: accounted → posted
    pub async fn post_entry(&self, entry_id: Uuid, posted_by: Option<Uuid>) -> AtlasResult<SubledgerJournalEntry> {
        let entry = self.repository.get_journal_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Journal entry {} not found", entry_id)
            ))?;

        if entry.status != "accounted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot post entry in '{}' status. Must be 'accounted'.",
                entry.status
            )));
        }

        info!("Posting SLA journal entry {}", entry.entry_number);
        self.repository.update_journal_entry_status(
            entry_id, "posted", None, None, posted_by, None,
        ).await
    }

    /// Reverse a posted journal entry
    /// Transition: posted → reversed
    pub async fn reverse_entry(&self, entry_id: Uuid, reason: &str, reversed_by: Option<Uuid>) -> AtlasResult<SubledgerJournalEntry> {
        let entry = self.repository.get_journal_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Journal entry {} not found", entry_id)
            ))?;

        if entry.status != "posted" && entry.status != "accounted" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot reverse entry in '{}' status. Must be 'accounted' or 'posted'.",
                entry.status
            )));
        }

        if reason.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Reversal reason is required".to_string(),
            ));
        }

        info!("Reversing SLA journal entry {} - reason: {}", entry.entry_number, reason);

        // Create a reversal SLA event
        let _event = self.repository.create_sla_event(
            entry.organization_id,
            &format!("REV-{}", &entry.entry_number[4..]),
            "reversal",
            &entry.source_application,
            &entry.source_transaction_type,
            entry.source_transaction_id,
            Some(entry_id),
            chrono::Utc::now().date_naive(),
            "processed",
            Some(&format!("Reversal: {}", reason)),
            None,
            reversed_by,
        ).await?;

        // Update original entry status
        self.repository.update_journal_entry_status(
            entry_id, "reversed", None, None, None, None,
        ).await
    }

    // ========================================================================
    // Transfer to GL
    // ========================================================================

    /// Transfer posted journal entries to the General Ledger
    /// Transition: posted → transferred (for each entry) + create GL transfer log
    pub async fn transfer_to_gl(
        &self,
        org_id: Uuid,
        from_period: Option<&str>,
        source_applications: Option<Vec<String>>,
        transferred_by: Option<Uuid>,
    ) -> AtlasResult<GlTransferLog> {
        // Get all posted entries eligible for transfer
        let entries = self.repository.list_journal_entries(
            org_id, Some("posted"), None, None, None, None,
        ).await?;

        // Filter by period and application if specified
        let eligible: Vec<SubledgerJournalEntry> = entries.into_iter()
            .filter(|e| {
                if let Some(period) = from_period {
                    e.period_name.as_deref() == Some(period)
                } else {
                    true
                }
            })
            .filter(|e| {
                if let Some(ref apps) = source_applications {
                    apps.contains(&e.source_application)
                } else {
                    true
                }
            })
            .collect();

        if eligible.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "No posted entries available for transfer".to_string(),
            ));
        }

        let transfer_number = format!("GLX-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
        let total_entries = eligible.len() as i32;
        let total_debit: f64 = eligible.iter()
            .map(|e| e.total_debit.parse::<f64>().unwrap_or(0.0))
            .sum();
        let total_credit: f64 = eligible.iter()
            .map(|e| e.total_credit.parse::<f64>().unwrap_or(0.0))
            .sum();
        let included_apps: Vec<String> = eligible.iter()
            .map(|e| e.source_application.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let entry_refs: Vec<serde_json::Value> = eligible.iter()
            .map(|e| serde_json::json!({
                "entry_id": e.id,
                "entry_number": e.entry_number,
            }))
            .collect();

        // Create transfer log
        let transfer_log = self.repository.create_transfer_log(
            org_id,
            &transfer_number,
            from_period,
            "completed",
            total_entries,
            &format!("{:.2}", total_debit),
            &format!("{:.2}", total_credit),
            serde_json::json!(included_apps),
            transferred_by,
            serde_json::json!(entry_refs),
        ).await?;

        // Update each entry to 'transferred' status
        for entry in &eligible {
            let _ = self.repository.update_journal_entry_status(
                entry.id, "transferred", None, None, None, None,
            ).await;

            // Update GL transfer fields
            // (In a real implementation, this would also create the GL journal entry)
        }

        info!("Transferred {} SLA entries to GL ({})", eligible.len(), transfer_number);

        Ok(transfer_log)
    }

    /// Get a transfer log
    pub async fn get_transfer_log(&self, id: Uuid) -> AtlasResult<Option<GlTransferLog>> {
        self.repository.get_transfer_log(id).await
    }

    /// List transfer logs
    pub async fn list_transfer_logs(&self, org_id: Uuid, status: Option<&str>) -> AtlasResult<Vec<GlTransferLog>> {
        self.repository.list_transfer_logs(org_id, status).await
    }

    // ========================================================================
    // SLA Events
    // ========================================================================

    /// List SLA events
    pub async fn list_sla_events(
        &self,
        org_id: Uuid,
        source_application: Option<&str>,
        event_type: Option<&str>,
    ) -> AtlasResult<Vec<SlaEvent>> {
        self.repository.list_sla_events(org_id, source_application, event_type).await
    }

    // ========================================================================
    // Derivation (Auto-Accounting)
    // ========================================================================

    /// Generate journal lines from a source transaction using derivation rules.
    /// This is the "auto-accounting" feature of Oracle Fusion SLA.
    pub async fn generate_journal_lines(
        &self,
        org_id: Uuid,
        entry_id: Uuid,
        transaction_attributes: &serde_json::Value,
    ) -> AtlasResult<Vec<SubledgerJournalLine>> {
        let entry = self.repository.get_journal_entry(entry_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Journal entry {} not found", entry_id)
            ))?;

        // Find the accounting method for this entry
        let method_id = entry.accounting_method_id
            .ok_or_else(|| AtlasError::ValidationFailed(
                "Journal entry has no accounting method; cannot auto-generate lines".to_string(),
            ))?;

        // Get all active derivation rules for the method
        let all_rules = self.repository.list_active_derivation_rules(
            org_id, method_id, "debit",
        ).await?;

        let credit_rules = self.repository.list_active_derivation_rules(
            org_id, method_id, "credit",
        ).await?;

        let mut lines = Vec::new();
        let mut line_num = 1;

        // Generate debit lines
        for rule in &all_rules {
            if let Some(account_code) = self.resolve_account_code(
                std::slice::from_ref(rule), "debit", transaction_attributes,
            ) {
                // For auto-generation, we use the amount from transaction attributes
                let amount = transaction_attributes.get("amount")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                if amount > 0.0 {
                    let line = self.repository.create_journal_line(
                        org_id, entry_id, line_num, "debit",
                        &account_code, None, Some(rule.id),
                        &format!("{:.2}", amount), &format!("{:.2}", amount),
                        &entry.currency_code, None, None,
                        None, None, None, None, None, None,
                        None, None, None, None, None,
                    ).await?;
                    lines.push(line);
                    line_num += 1;
                }
            }
        }

        // Generate credit lines
        for rule in &credit_rules {
            if let Some(account_code) = self.resolve_account_code(
                std::slice::from_ref(rule), "credit", transaction_attributes,
            ) {
                let amount = transaction_attributes.get("amount")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                if amount > 0.0 {
                    let line = self.repository.create_journal_line(
                        org_id, entry_id, line_num, "credit",
                        &account_code, None, Some(rule.id),
                        &format!("{:.2}", amount), &format!("{:.2}", amount),
                        &entry.currency_code, None, None,
                        None, None, None, None, None, None,
                        None, None, None, None, None,
                    ).await?;
                    lines.push(line);
                    line_num += 1;
                }
            }
        }

        // Recalculate balances
        self.recalculate_entry_balances(entry_id).await?;

        info!("Generated {} journal lines for entry {}", lines.len(), entry.entry_number);
        Ok(lines)
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get SLA dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<SlaDashboardSummary> {
        let entries = self.repository.list_journal_entries(
            org_id, None, None, None, None, None,
        ).await?;

        let mut draft_count = 0i32;
        let mut accounted_count = 0i32;
        let mut posted_count = 0i32;
        let mut transferred_count = 0i32;
        let mut reversed_count = 0i32;
        let mut error_count = 0i32;
        let mut pending_transfer_count = 0i32;
        let mut unbalanced_count = 0i32;
        let mut total_debit = 0.0f64;
        let mut total_credit = 0.0f64;
        let mut by_status: std::collections::HashMap<String, (i32, f64, f64)> = std::collections::HashMap::new();
        let mut by_app: std::collections::HashMap<String, i32> = std::collections::HashMap::new();

        for entry in &entries {
            total_debit += entry.total_debit.parse::<f64>().unwrap_or(0.0);
            total_credit += entry.total_credit.parse::<f64>().unwrap_or(0.0);

            match entry.status.as_str() {
                "draft" => draft_count += 1,
                "accounted" => accounted_count += 1,
                "posted" => posted_count += 1,
                "transferred" => transferred_count += 1,
                "reversed" => reversed_count += 1,
                "error" => error_count += 1,
                _ => {}
            }

            if (entry.status == "posted" || entry.status == "accounted")
                && entry.gl_transfer_status == "pending" {
                    pending_transfer_count += 1;
                }

            if !entry.is_balanced {
                unbalanced_count += 1;
            }

            let debit = entry.total_debit.parse::<f64>().unwrap_or(0.0);
            let credit = entry.total_credit.parse::<f64>().unwrap_or(0.0);
            by_status.entry(entry.status.clone())
                .and_modify(|(c, d, cr)| { *c += 1; *d += debit; *cr += credit; })
                .or_insert((1, debit, credit));

            *by_app.entry(entry.source_application.clone()).or_insert(0) += 1;
        }

        let entries_by_status: serde_json::Value = by_status.into_iter()
            .map(|(k, (c, d, cr))| serde_json::json!({
                "status": k, "count": c,
                "total_debit": format!("{:.2}", d),
                "total_credit": format!("{:.2}", cr),
            }))
            .collect();

        let entries_by_application: serde_json::Value = by_app.into_iter()
            .map(|(k, c)| serde_json::json!({"application": k, "count": c}))
            .collect();

        Ok(SlaDashboardSummary {
            total_entries: entries.len() as i32,
            draft_count,
            accounted_count,
            posted_count,
            transferred_count,
            reversed_count,
            error_count,
            total_debit: format!("{:.2}", total_debit),
            total_credit: format!("{:.2}", total_credit),
            entries_by_application,
            entries_by_status,
            pending_transfer_count,
            unbalanced_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_repos::MockSubledgerAccountingRepository;

    fn create_engine() -> SubledgerAccountingEngine {
        SubledgerAccountingEngine::new(Arc::new(MockSubledgerAccountingRepository))
    }

    #[test]
    fn test_valid_applications() {
        assert!(VALID_APPLICATIONS.contains(&"payables"));
        assert!(VALID_APPLICATIONS.contains(&"receivables"));
        assert!(VALID_APPLICATIONS.contains(&"expenses"));
        assert!(VALID_APPLICATIONS.contains(&"assets"));
        assert!(VALID_APPLICATIONS.contains(&"projects"));
        assert!(VALID_APPLICATIONS.contains(&"general"));
    }

    #[test]
    fn test_valid_event_classes() {
        assert!(VALID_EVENT_CLASSES.contains(&"create"));
        assert!(VALID_EVENT_CLASSES.contains(&"update"));
        assert!(VALID_EVENT_CLASSES.contains(&"cancel"));
        assert!(VALID_EVENT_CLASSES.contains(&"reverse"));
    }

    #[test]
    fn test_valid_entry_statuses() {
        assert!(VALID_ENTRY_STATUSES.contains(&"draft"));
        assert!(VALID_ENTRY_STATUSES.contains(&"accounted"));
        assert!(VALID_ENTRY_STATUSES.contains(&"posted"));
        assert!(VALID_ENTRY_STATUSES.contains(&"transferred"));
        assert!(VALID_ENTRY_STATUSES.contains(&"reversed"));
        assert!(VALID_ENTRY_STATUSES.contains(&"error"));
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"debit"));
        assert!(VALID_LINE_TYPES.contains(&"credit"));
        assert!(VALID_LINE_TYPES.contains(&"tax"));
        assert!(VALID_LINE_TYPES.contains(&"discount"));
        assert!(VALID_LINE_TYPES.contains(&"rounding"));
    }

    #[test]
    fn test_valid_derivation_types() {
        assert!(VALID_DERIVATION_TYPES.contains(&"constant"));
        assert!(VALID_DERIVATION_TYPES.contains(&"lookup"));
        assert!(VALID_DERIVATION_TYPES.contains(&"formula"));
    }

    #[test]
    fn test_resolve_account_code_constant() {
        let engine = create_engine();

        let rule = AccountingDerivationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            accounting_method_id: Uuid::new_v4(),
            code: "PAY_DEBIT".to_string(),
            name: "Payables Debit".to_string(),
            description: None,
            line_type: "debit".to_string(),
            priority: 10,
            conditions: serde_json::json!({}),
            source_field: None,
            derivation_type: "constant".to_string(),
            fixed_account_code: Some("2100".to_string()),
            account_derivation_lookup: serde_json::json!({}),
            formula_expression: None,
            sequence: 10,
            is_active: true,
            effective_from: None,
            effective_to: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = engine.resolve_account_code(
            &[rule], "debit", &serde_json::json!({}),
        );
        assert_eq!(result, Some("2100".to_string()));
    }

    #[test]
    fn test_resolve_account_code_lookup() {
        let engine = create_engine();

        let rule = AccountingDerivationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            accounting_method_id: Uuid::new_v4(),
            code: "EXP_DEBIT".to_string(),
            name: "Expense Debit by Category".to_string(),
            description: None,
            line_type: "debit".to_string(),
            priority: 10,
            conditions: serde_json::json!({}),
            source_field: Some("expense_category".to_string()),
            derivation_type: "lookup".to_string(),
            fixed_account_code: None,
            account_derivation_lookup: serde_json::json!({
                "Travel": "6100",
                "Meals": "6200",
                "Office": "6300",
            }),
            formula_expression: None,
            sequence: 10,
            is_active: true,
            effective_from: None,
            effective_to: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = engine.resolve_account_code(
            std::slice::from_ref(&rule), "debit", &serde_json::json!({"expense_category": "Travel"}),
        );
        assert_eq!(result, Some("6100".to_string()));

        let result = engine.resolve_account_code(
            std::slice::from_ref(&rule), "debit", &serde_json::json!({"expense_category": "Meals"}),
        );
        assert_eq!(result, Some("6200".to_string()));

        // Unknown category returns None
        let result = engine.resolve_account_code(
            &[rule], "debit", &serde_json::json!({"expense_category": "Unknown"}),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_account_code_with_conditions() {
        let engine = create_engine();

        // Rule with matching conditions
        let rule_matching = AccountingDerivationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            accounting_method_id: Uuid::new_v4(),
            code: "AP_DEBIT".to_string(),
            name: "AP Debit".to_string(),
            description: None,
            line_type: "debit".to_string(),
            priority: 10,
            conditions: serde_json::json!({"category": "Travel"}),
            source_field: None,
            derivation_type: "constant".to_string(),
            fixed_account_code: Some("6100".to_string()),
            account_derivation_lookup: serde_json::json!({}),
            formula_expression: None,
            sequence: 10,
            is_active: true,
            effective_from: None,
            effective_to: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Conditions match - should return account code
        let result = engine.resolve_account_code(
            std::slice::from_ref(&rule_matching), "debit", &serde_json::json!({"category": "Travel"}),
        );
        assert_eq!(result, Some("6100".to_string()));

        // Rule with non-matching conditions
        let rule_non_matching = AccountingDerivationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            accounting_method_id: Uuid::new_v4(),
            code: "AP_DEBIT_RESTRICTED".to_string(),
            name: "AP Debit Restricted".to_string(),
            description: None,
            line_type: "debit".to_string(),
            priority: 5,
            conditions: serde_json::json!({"category": "Meals"}),
            source_field: None,
            derivation_type: "constant".to_string(),
            fixed_account_code: Some("6200".to_string()),
            account_derivation_lookup: serde_json::json!({}),
            formula_expression: None,
            sequence: 5,
            is_active: true,
            effective_from: None,
            effective_to: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Conditions don't match - should skip this rule and return None (no fallback)
        let result = engine.resolve_account_code(
            &[rule_non_matching], "debit", &serde_json::json!({"category": "Travel"}),
        );
        assert!(result.is_none(),
            "Expected None when conditions don't match, got {:?}", result);

        // Empty conditions always match
        let rule_empty = AccountingDerivationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            accounting_method_id: Uuid::new_v4(),
            code: "DEFAULT_DEBIT".to_string(),
            name: "Default Debit".to_string(),
            description: None,
            line_type: "debit".to_string(),
            priority: 20,
            conditions: serde_json::json!({}),
            source_field: None,
            derivation_type: "constant".to_string(),
            fixed_account_code: Some("9999".to_string()),
            account_derivation_lookup: serde_json::json!({}),
            formula_expression: None,
            sequence: 20,
            is_active: true,
            effective_from: None,
            effective_to: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = engine.resolve_account_code(
            &[rule_empty], "debit", &serde_json::json!({"category": "Travel"}),
        );
        assert_eq!(result, Some("9999".to_string()));
    }

    #[test]
    fn test_resolve_account_code_priority_ordering() {
        let engine = create_engine();

        let low_priority = AccountingDerivationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            accounting_method_id: Uuid::new_v4(),
            code: "DEFAULT_DEBIT".to_string(),
            name: "Default Debit".to_string(),
            description: None,
            line_type: "debit".to_string(),
            priority: 20,
            conditions: serde_json::json!({}),
            source_field: None,
            derivation_type: "constant".to_string(),
            fixed_account_code: Some("9999".to_string()),
            account_derivation_lookup: serde_json::json!({}),
            formula_expression: None,
            sequence: 20,
            is_active: true,
            effective_from: None,
            effective_to: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let high_priority = AccountingDerivationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            accounting_method_id: Uuid::new_v4(),
            code: "SPECIFIC_DEBIT".to_string(),
            name: "Specific Debit".to_string(),
            description: None,
            line_type: "debit".to_string(),
            priority: 5,
            conditions: serde_json::json!({}),
            source_field: None,
            derivation_type: "constant".to_string(),
            fixed_account_code: Some("2100".to_string()),
            account_derivation_lookup: serde_json::json!({}),
            formula_expression: None,
            sequence: 5,
            is_active: true,
            effective_from: None,
            effective_to: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Higher priority (lower number) should be returned first
        let result = engine.resolve_account_code(
            &[low_priority.clone(), high_priority.clone()], "debit", &serde_json::json!({}),
        );
        assert_eq!(result, Some("2100".to_string()));
    }

    #[test]
    fn test_resolve_account_code_returns_none_for_wrong_line_type() {
        let engine = create_engine();

        let rule = AccountingDerivationRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            accounting_method_id: Uuid::new_v4(),
            code: "PAY_DEBIT".to_string(),
            name: "Payables Debit".to_string(),
            description: None,
            line_type: "debit".to_string(),
            priority: 10,
            conditions: serde_json::json!({}),
            source_field: None,
            derivation_type: "constant".to_string(),
            fixed_account_code: Some("2100".to_string()),
            account_derivation_lookup: serde_json::json!({}),
            formula_expression: None,
            sequence: 10,
            is_active: true,
            effective_from: None,
            effective_to: None,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Looking for credit rules but only have debit
        let result = engine.resolve_account_code(
            &[rule], "credit", &serde_json::json!({}),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_conditions_basic() {
        // Empty conditions always match
        let result = SubledgerAccountingEngine::evaluate_conditions(
            &serde_json::json!({}),
            &serde_json::json!({"amount": 100}),
        );
        assert!(result);

        // Null conditions always match
        let result = SubledgerAccountingEngine::evaluate_conditions(
            &serde_json::Value::Null,
            &serde_json::json!({}),
        );
        assert!(result);

        // Matching string condition
        let result = SubledgerAccountingEngine::evaluate_conditions(
            &serde_json::json!({"category": "Travel"}),
            &serde_json::json!({"category": "Travel", "amount": 100}),
        );
        assert!(result);

        // Non-matching string condition
        let result = SubledgerAccountingEngine::evaluate_conditions(
            &serde_json::json!({"category": "Travel"}),
            &serde_json::json!({"category": "Meals", "amount": 100}),
        );
        assert!(!result);

        // Missing field in attributes
        let result = SubledgerAccountingEngine::evaluate_conditions(
            &serde_json::json!({"category": "Travel"}),
            &serde_json::json!({"amount": 100}),
        );
        assert!(!result);

        // Matching number condition
        let result = SubledgerAccountingEngine::evaluate_conditions(
            &serde_json::json!({"amount": 100.0}),
            &serde_json::json!({"amount": 100.0}),
        );
        assert!(result);

        // Matching boolean condition
        let result = SubledgerAccountingEngine::evaluate_conditions(
            &serde_json::json!({"is_recurring": true}),
            &serde_json::json!({"is_recurring": true}),
        );
        assert!(result);
    }

    #[test]
    fn test_dashboard_summary_empty() {
        let engine = create_engine();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let summary = rt.block_on(async {
            engine.get_dashboard_summary(Uuid::new_v4()).await.unwrap()
        });

        assert_eq!(summary.total_entries, 0);
        assert_eq!(summary.draft_count, 0);
        assert_eq!(summary.posted_count, 0);
        assert_eq!(summary.error_count, 0);
        assert_eq!(summary.pending_transfer_count, 0);
        assert_eq!(summary.unbalanced_count, 0);
    }

    #[test]
    fn test_invalid_application_rejected() {
        let engine = create_engine();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            engine.create_accounting_method(
                Uuid::new_v4(), "TEST", "Test", None,
                "invalid_app", "invoice", None, None, None, None,
                None, None, None, None, None, None, None,
            ).await
        });
        assert!(result.is_err());
        if let Err(AtlasError::ValidationFailed(msg)) = result {
            assert!(msg.contains("Invalid application"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[test]
    fn test_invalid_event_class_rejected() {
        let engine = create_engine();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            engine.create_accounting_method(
                Uuid::new_v4(), "TEST", "Test", None,
                "payables", "invoice", Some("invalid"), None, None, None,
                None, None, None, None, None, None, None,
            ).await
        });
        assert!(result.is_err());
        if let Err(AtlasError::ValidationFailed(msg)) = result {
            assert!(msg.contains("Invalid event_class"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[test]
    fn test_invalid_derivation_type_rejected() {
        let engine = create_engine();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            engine.create_derivation_rule(
                Uuid::new_v4(), Uuid::new_v4(), "TEST", "Test", None,
                "debit", 10, serde_json::json!({}), None,
                "invalid_type", None, serde_json::json!({}), None,
                10, None, None, None,
            ).await
        });
        assert!(result.is_err());
        if let Err(AtlasError::ValidationFailed(msg)) = result {
            assert!(msg.contains("Invalid derivation_type"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[test]
    fn test_constant_derivation_requires_account_code() {
        let engine = create_engine();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            engine.create_derivation_rule(
                Uuid::new_v4(), Uuid::new_v4(), "TEST", "Test", None,
                "debit", 10, serde_json::json!({}), None,
                "constant", None, serde_json::json!({}), None,  // missing fixed_account_code
                10, None, None, None,
            ).await
        });
        assert!(result.is_err());
        if let Err(AtlasError::ValidationFailed(msg)) = result {
            assert!(msg.contains("Constant derivation type requires fixed_account_code"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }
}