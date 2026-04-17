//! Expense Engine Implementation
//!
//! Manages expense categories, policies, expense reports with line items,
//! per-diem calculation, mileage calculation, and policy validation.
//!
//! Oracle Fusion Cloud ERP equivalent: Expenses > Expense Reports, Categories, Policies

use atlas_shared::{
    ExpenseCategory, ExpensePolicy, ExpenseReport, ExpenseLine,
    AtlasError, AtlasResult,
};
use super::ExpenseRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid expense types
const VALID_EXPENSE_TYPES: &[&str] = &[
    "expense", "per_diem", "mileage", "credit_card",
];

/// Valid violation actions
const VALID_VIOLATION_ACTIONS: &[&str] = &[
    "warn", "block", "require_justification",
];

/// Valid report statuses for transitions
const VALID_STATUSES: &[&str] = &[
    "draft", "submitted", "approved", "rejected", "reimbursed", "cancelled",
];

/// Expense engine for managing expense reports, categories, and policies
pub struct ExpenseEngine {
    repository: Arc<dyn ExpenseRepository>,
}

impl ExpenseEngine {
    pub fn new(repository: Arc<dyn ExpenseRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Expense Category Management
    // ========================================================================

    /// Create a new expense category
    pub async fn create_category(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        receipt_required: bool,
        receipt_threshold: Option<&str>,
        is_per_diem: bool,
        default_per_diem_rate: Option<&str>,
        is_mileage: bool,
        default_mileage_rate: Option<&str>,
        expense_account_code: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpenseCategory> {
        let code_upper = code.to_uppercase();
        if code_upper.is_empty() || code_upper.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Expense category code must be 1-50 characters".to_string(),
            ));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Expense category name is required".to_string(),
            ));
        }

        info!("Creating expense category '{}' ({}) for org {}", code_upper, name, org_id);

        self.repository.create_category(
            org_id, &code_upper, name, description,
            receipt_required, receipt_threshold,
            is_per_diem, default_per_diem_rate,
            is_mileage, default_mileage_rate,
            expense_account_code, created_by,
        ).await
    }

    /// Get an expense category by code
    pub async fn get_category(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ExpenseCategory>> {
        self.repository.get_category(org_id, &code.to_uppercase()).await
    }

    /// List all categories for an organization
    pub async fn list_categories(&self, org_id: Uuid) -> AtlasResult<Vec<ExpenseCategory>> {
        self.repository.list_categories(org_id).await
    }

    /// Deactivate a category
    pub async fn delete_category(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deactivating expense category '{}' for org {}", code, org_id);
        self.repository.delete_category(org_id, &code.to_uppercase()).await
    }

    // ========================================================================
    // Expense Policy Management
    // ========================================================================

    /// Create a new expense policy
    pub async fn create_policy(
        &self,
        org_id: Uuid,
        name: &str,
        description: Option<&str>,
        category_code: Option<&str>,
        min_amount: Option<&str>,
        max_amount: Option<&str>,
        daily_limit: Option<&str>,
        report_limit: Option<&str>,
        requires_approval_on_violation: bool,
        violation_action: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpensePolicy> {
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Policy name is required".to_string(),
            ));
        }
        if !VALID_VIOLATION_ACTIONS.contains(&violation_action) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid violation_action '{}'. Must be one of: {}", violation_action, VALID_VIOLATION_ACTIONS.join(", ")
            )));
        }

        // Resolve category code to ID if provided
        let category_id = if let Some(cc) = category_code {
            let cat = self.get_category(org_id, cc).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Expense category '{}' not found", cc)
                ))?;
            Some(cat.id)
        } else {
            None
        };

        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }

        info!("Creating expense policy '{}' for org {}", name, org_id);

        self.repository.create_policy(
            org_id, name, description, category_id,
            min_amount, max_amount, daily_limit, report_limit,
            requires_approval_on_violation, violation_action,
            effective_from, effective_to, created_by,
        ).await
    }

    /// List policies for an organization, optionally filtered by category
    pub async fn list_policies(&self, org_id: Uuid, category_code: Option<&str>) -> AtlasResult<Vec<ExpensePolicy>> {
        let category_id = if let Some(cc) = category_code {
            let cat = self.get_category(org_id, cc).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Expense category '{}' not found", cc)
                ))?;
            Some(cat.id)
        } else {
            None
        };
        self.repository.list_policies(org_id, category_id).await
    }

    /// Delete a policy
    pub async fn delete_policy(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deactivating expense policy {}", id);
        self.repository.delete_policy(id).await
    }

    // ========================================================================
    // Expense Report Management
    // ========================================================================

    /// Create a new expense report
    pub async fn create_report(
        &self,
        org_id: Uuid,
        report_number: &str,
        title: &str,
        description: Option<&str>,
        employee_id: Uuid,
        employee_name: Option<&str>,
        department_id: Option<Uuid>,
        purpose: Option<&str>,
        project_id: Option<Uuid>,
        currency_code: &str,
        trip_start_date: Option<chrono::NaiveDate>,
        trip_end_date: Option<chrono::NaiveDate>,
        cost_center: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpenseReport> {
        if report_number.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Report number is required".to_string(),
            ));
        }
        if title.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Report title is required".to_string(),
            ));
        }
        if let (Some(start), Some(end)) = (trip_start_date, trip_end_date) {
            if start > end {
                return Err(AtlasError::ValidationFailed(
                    "Trip start date must be before end date".to_string(),
                ));
            }
        }

        info!("Creating expense report '{}' for org {} employee {}", report_number, org_id, employee_id);

        self.repository.create_report(
            org_id, report_number, title, description,
            employee_id, employee_name, department_id, purpose,
            project_id, currency_code,
            trip_start_date, trip_end_date, cost_center,
            created_by,
        ).await
    }

    /// Get an expense report by ID
    pub async fn get_report(&self, id: Uuid) -> AtlasResult<Option<ExpenseReport>> {
        self.repository.get_report(id).await
    }

    /// Get an expense report by number
    pub async fn get_report_by_number(&self, org_id: Uuid, report_number: &str) -> AtlasResult<Option<ExpenseReport>> {
        self.repository.get_report_by_number(org_id, report_number).await
    }

    /// List expense reports with optional filters
    pub async fn list_reports(
        &self,
        org_id: Uuid,
        employee_id: Option<Uuid>,
        status: Option<&str>,
    ) -> AtlasResult<Vec<ExpenseReport>> {
        if let Some(s) = status {
            if !VALID_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_STATUSES.join(", ")
                )));
            }
        }
        self.repository.list_reports(org_id, employee_id, status).await
    }

    // ========================================================================
    // Expense Report Workflow
    // ========================================================================

    /// Submit an expense report for approval
    /// Only draft reports can be submitted
    pub async fn submit_report(&self, report_id: Uuid) -> AtlasResult<ExpenseReport> {
        let report = self.repository.get_report(report_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Expense report {} not found", report_id)
            ))?;

        if report.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot submit report in '{}' status. Must be 'draft'.", report.status)
            ));
        }

        // Check report has at least one line
        let lines = self.repository.list_lines_by_report(report_id).await?;
        if lines.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Expense report must have at least one expense line before submission".to_string(),
            ));
        }

        info!("Submitting expense report {}", report_id);
        self.repository.update_report_status(report_id, "submitted", None, None, None).await
    }

    /// Approve an expense report
    pub async fn approve_report(
        &self,
        report_id: Uuid,
        approved_by: Uuid,
    ) -> AtlasResult<ExpenseReport> {
        let report = self.repository.get_report(report_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Expense report {} not found", report_id)
            ))?;

        if report.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot approve report in '{}' status. Must be 'submitted'.", report.status)
            ));
        }

        info!("Approving expense report {} by {}", report_id, approved_by);
        self.repository.update_report_status(report_id, "approved", Some(approved_by), None, None).await
    }

    /// Reject an expense report
    pub async fn reject_report(
        &self,
        report_id: Uuid,
        rejected_by: Uuid,
        reason: Option<&str>,
    ) -> AtlasResult<ExpenseReport> {
        let report = self.repository.get_report(report_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Expense report {} not found", report_id)
            ))?;

        if report.status != "submitted" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reject report in '{}' status. Must be 'submitted'.", report.status)
            ));
        }

        info!("Rejecting expense report {} by {}", report_id, rejected_by);
        self.repository.update_report_status(report_id, "rejected", Some(rejected_by), reason, None).await
    }

    /// Mark an expense report as reimbursed
    pub async fn reimburse_report(
        &self,
        report_id: Uuid,
    ) -> AtlasResult<ExpenseReport> {
        let report = self.repository.get_report(report_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Expense report {} not found", report_id)
            ))?;

        if report.status != "approved" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot reimburse report in '{}' status. Must be 'approved'.", report.status)
            ));
        }

        info!("Reimbursing expense report {}", report_id);
        self.repository.update_report_status(
            report_id, "reimbursed", report.approved_by, None, Some(chrono::Utc::now()),
        ).await
    }

    /// Cancel a draft expense report
    pub async fn cancel_report(&self, report_id: Uuid) -> AtlasResult<ExpenseReport> {
        let report = self.repository.get_report(report_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Expense report {} not found", report_id)
            ))?;

        if report.status != "draft" {
            return Err(AtlasError::WorkflowError(
                format!("Cannot cancel report in '{}' status. Must be 'draft'.", report.status)
            ));
        }

        info!("Cancelling expense report {}", report_id);
        self.repository.update_report_status(report_id, "cancelled", None, None, None).await
    }

    // ========================================================================
    // Expense Line Management
    // ========================================================================

    /// Add an expense line to a report
    pub async fn add_line(
        &self,
        org_id: Uuid,
        report_id: Uuid,
        expense_type: &str,
        expense_date: chrono::NaiveDate,
        amount: &str,
        description: Option<&str>,
        category_code: Option<&str>,
        original_currency: Option<&str>,
        original_amount: Option<&str>,
        exchange_rate: Option<&str>,
        is_reimbursable: Option<bool>,
        has_receipt: Option<bool>,
        receipt_reference: Option<&str>,
        merchant_name: Option<&str>,
        location: Option<&str>,
        attendees: Option<serde_json::Value>,
        per_diem_days: Option<f64>,
        per_diem_rate: Option<&str>,
        mileage_distance: Option<f64>,
        mileage_rate: Option<&str>,
        mileage_unit: Option<&str>,
        mileage_from: Option<&str>,
        mileage_to: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ExpenseLine> {
        // Validate report exists and is in draft status
        let report = self.repository.get_report(report_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Expense report {} not found", report_id)
            ))?;

        if report.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only add lines to a draft expense report".to_string(),
            ));
        }

        if !VALID_EXPENSE_TYPES.contains(&expense_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid expense_type '{}'. Must be one of: {}", expense_type, VALID_EXPENSE_TYPES.join(", ")
            )));
        }

        let amount_val: f64 = amount.parse().map_err(|_| AtlasError::ValidationFailed(
            "Amount must be a valid number".to_string(),
        ))?;
        if amount_val < 0.0 {
            return Err(AtlasError::ValidationFailed(
                "Amount must be non-negative".to_string(),
            ));
        }

        // Resolve category
        let mut category_name: Option<String> = None;
        let mut category_id: Option<Uuid> = None;
        let resolved_expense_type = expense_type.to_string();
        let mut resolved_amount = amount_val;
        let mut expense_account_code: Option<String> = None;

        if let Some(cc) = category_code {
            let cat = self.get_category(org_id, cc).await?
                .ok_or_else(|| AtlasError::EntityNotFound(
                    format!("Expense category '{}' not found", cc)
                ))?;
            category_id = Some(cat.id);
            category_name = Some(cat.name);
            expense_account_code = cat.expense_account_code;

            // Auto-calculate per-diem
            if cat.is_per_diem && expense_type == "per_diem" {
                let days = per_diem_days.unwrap_or(1.0);
                let rate: f64 = per_diem_rate
                    .map(|r| r.parse().unwrap_or(0.0))
                    .or_else(|| cat.default_per_diem_rate.as_ref().map(|r| r.parse().unwrap_or(0.0)))
                    .unwrap_or(0.0);
                resolved_amount = days * rate;
            }

            // Auto-calculate mileage
            if cat.is_mileage && expense_type == "mileage" {
                let distance = mileage_distance.unwrap_or(0.0);
                let rate: f64 = mileage_rate
                    .map(|r| r.parse().unwrap_or(0.0))
                    .or_else(|| cat.default_mileage_rate.as_ref().map(|r| r.parse().unwrap_or(0.0)))
                    .unwrap_or(0.0);
                resolved_amount = distance * rate;
            }
        }

        // Determine line number
        let existing_lines = self.repository.list_lines_by_report(report_id).await?;
        let line_number = (existing_lines.len() + 1) as i32;

        // Validate against expense policies
        let (policy_violation, policy_violation_message) =
            self.validate_line_against_policies(org_id, category_id.as_ref(), resolved_amount, &expense_date).await?;

        let is_reimb = is_reimbursable.unwrap_or(true);

        info!("Adding expense line to report {} (line {})", report_id, line_number);

        let line = self.repository.create_line(
            org_id, report_id, line_number,
            category_id, category_name.as_deref(), &resolved_expense_type,
            description, expense_date,
            &format!("{:.2}", resolved_amount),
            original_currency, original_amount, exchange_rate,
            is_reimb,
            has_receipt.unwrap_or(false), receipt_reference,
            merchant_name, location, attendees,
            per_diem_days, per_diem_rate,
            mileage_distance, mileage_rate, mileage_unit,
            mileage_from, mileage_to,
            policy_violation, policy_violation_message.as_deref(),
            expense_account_code.as_deref(), created_by,
        ).await?;

        // Recalculate report totals
        self.recalculate_report_totals(report_id).await?;

        Ok(line)
    }

    /// Delete an expense line from a report
    pub async fn delete_line(&self, report_id: Uuid, line_id: Uuid) -> AtlasResult<()> {
        let report = self.repository.get_report(report_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Expense report {} not found", report_id)
            ))?;

        if report.status != "draft" {
            return Err(AtlasError::WorkflowError(
                "Can only delete lines from a draft expense report".to_string(),
            ));
        }

        self.repository.delete_line(line_id).await?;

        // Recalculate report totals
        self.recalculate_report_totals(report_id).await?;

        Ok(())
    }

    /// List all lines for a report
    pub async fn list_lines(&self, report_id: Uuid) -> AtlasResult<Vec<ExpenseLine>> {
        self.repository.list_lines_by_report(report_id).await
    }

    // ========================================================================
    // Policy Validation
    // ========================================================================

    /// Validate an expense line against applicable policies
    async fn validate_line_against_policies(
        &self,
        org_id: Uuid,
        category_id: Option<&Uuid>,
        amount: f64,
        _expense_date: &chrono::NaiveDate,
    ) -> AtlasResult<(bool, Option<String>)> {
        let policies = self.repository.list_policies(org_id, category_id.copied()).await?;

        let mut violations = Vec::new();
        for policy in &policies {
            if !policy.is_active {
                continue;
            }

            // Check max amount
            if let Some(max_str) = &policy.max_amount {
                let max: f64 = max_str.parse().unwrap_or(f64::MAX);
                if amount > max {
                    violations.push(format!(
                        "Amount {:.2} exceeds policy '{}' maximum of {}",
                        amount, policy.name, max_str
                    ));
                }
            }

            // Check min amount
            if let Some(min_str) = &policy.min_amount {
                let min: f64 = min_str.parse().unwrap_or(0.0);
                if amount < min {
                    violations.push(format!(
                        "Amount {:.2} is below policy '{}' minimum of {}",
                        amount, policy.name, min_str
                    ));
                }
            }
        }

        if violations.is_empty() {
            Ok((false, None))
        } else {
            Ok((true, Some(violations.join("; "))))
        }
    }

    /// Recalculate report totals from expense lines
    async fn recalculate_report_totals(&self, report_id: Uuid) -> AtlasResult<()> {
        let lines = self.repository.list_lines_by_report(report_id).await?;

        let mut total: f64 = 0.0;
        let mut reimbursable: f64 = 0.0;
        let mut receipt_required: f64 = 0.0;
        let mut receipt_count: i32 = 0;

        for line in &lines {
            let amount: f64 = line.amount.parse().unwrap_or(0.0);
            total += amount;
            if line.is_reimbursable {
                reimbursable += amount;
            }
            if line.has_receipt {
                receipt_count += 1;
            }
            // Check if receipt is required for this amount
            if let Some(cat_id) = line.expense_category_id {
                if let Ok(Some(cat)) = self.repository.get_category_by_id(cat_id).await {
                    if cat.receipt_required {
                        if let Some(threshold_str) = &cat.receipt_threshold {
                            let threshold: f64 = threshold_str.parse().unwrap_or(0.0);
                            if amount >= threshold {
                                receipt_required += amount;
                            }
                        } else {
                            receipt_required += amount;
                        }
                    }
                }
            }
        }

        self.repository.update_report_totals(
            report_id,
            &format!("{:.2}", total),
            &format!("{:.2}", reimbursable),
            &format!("{:.2}", receipt_required),
            receipt_count,
        ).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_expense_types() {
        assert!(VALID_EXPENSE_TYPES.contains(&"expense"));
        assert!(VALID_EXPENSE_TYPES.contains(&"per_diem"));
        assert!(VALID_EXPENSE_TYPES.contains(&"mileage"));
        assert!(VALID_EXPENSE_TYPES.contains(&"credit_card"));
    }

    #[test]
    fn test_valid_violation_actions() {
        assert!(VALID_VIOLATION_ACTIONS.contains(&"warn"));
        assert!(VALID_VIOLATION_ACTIONS.contains(&"block"));
        assert!(VALID_VIOLATION_ACTIONS.contains(&"require_justification"));
    }

    #[test]
    fn test_valid_statuses() {
        assert!(VALID_STATUSES.contains(&"draft"));
        assert!(VALID_STATUSES.contains(&"submitted"));
        assert!(VALID_STATUSES.contains(&"approved"));
        assert!(VALID_STATUSES.contains(&"rejected"));
        assert!(VALID_STATUSES.contains(&"reimbursed"));
        assert!(VALID_STATUSES.contains(&"cancelled"));
    }
}
