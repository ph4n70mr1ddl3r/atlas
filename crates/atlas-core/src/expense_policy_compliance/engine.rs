//! Expense Policy Compliance Engine
//!
//! Manages expense policy rule lifecycle, compliance audit evaluation,
//! violation tracking, and audit lifecycle management.
//!
//! Oracle Fusion Cloud ERP equivalent: Expenses > Policies > Expense Policy Compliance

use atlas_shared::{
    ExpensePolicyRule, ExpenseComplianceAudit, ExpenseComplianceViolation,
    ExpenseComplianceDashboard,
    AtlasError, AtlasResult,
};
use super::ExpensePolicyComplianceRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid rule types for expense policy rules
const VALID_RULE_TYPES: &[&str] = &[
    "amount_limit", "daily_limit", "category_limit", "receipt_required",
    "time_restriction", "duplicate_check", "approval_required", "per_diem_override",
];

/// Valid severity levels
const VALID_SEVERITIES: &[&str] = &["warning", "violation", "block"];

/// Valid evaluation scopes
const VALID_EVALUATION_SCOPES: &[&str] = &["per_line", "per_day", "per_report", "per_trip"];

/// Valid expense categories
const VALID_EXPENSE_CATEGORIES: &[&str] = &[
    "airfare", "hotel", "meals", "transportation", "entertainment",
    "office_supplies", "telecommunications", "training", "other", "all",
];

/// Valid rule statuses
const VALID_RULE_STATUSES: &[&str] = &["draft", "active", "inactive"];

/// Valid audit triggers
const VALID_AUDIT_TRIGGERS: &[&str] = &[
    "automatic", "random_sample", "high_amount", "policy_violation", "manual",
];

/// Valid audit statuses
const VALID_AUDIT_STATUSES: &[&str] = &["pending", "in_review", "completed", "escalated"];

/// Valid risk levels
const VALID_RISK_LEVELS: &[&str] = &["low", "medium", "high", "critical"];

/// Valid violation resolution statuses
const VALID_RESOLUTION_STATUSES: &[&str] = &[
    "open", "justified", "adjusted", "upheld", "escalated",
];

/// Calculate compliance score based on violations, warnings, and blocks.
/// Score is 0-100, starting at 100 and applying penalties.
pub fn calculate_compliance_score(
    total_lines: i32,
    violations_count: i32,
    warnings_count: i32,
    blocks_count: i32,
) -> f64 {
    if total_lines == 0 {
        return 100.0;
    }

    let mut score = 100.0;

    // Each block costs 15 points
    score -= (blocks_count as f64) * 15.0;
    // Each violation costs 10 points
    score -= (violations_count as f64) * 10.0;
    // Each warning costs 3 points
    score -= (warnings_count as f64) * 3.0;

    score.clamp(0.0, 100.0)
}

/// Determine risk level from compliance score
pub fn determine_risk_level(score: f64) -> &'static str {
    if score >= 80.0 {
        "low"
    } else if score >= 60.0 {
        "medium"
    } else if score >= 40.0 {
        "high"
    } else {
        "critical"
    }
}

/// Check if an expense amount violates an amount limit rule
pub fn evaluate_amount_limit(
    expense_amount: f64,
    threshold_amount: Option<f64>,
    maximum_amount: Option<f64>,
) -> Option<(String, f64)> {
    // Check maximum amount
    if let Some(max) = maximum_amount {
        if expense_amount > max {
            let excess = expense_amount - max;
            return Some((
                format!("Expense amount {:.2} exceeds maximum allowed {:.2}", expense_amount, max),
                excess,
            ));
        }
    }
    // Check threshold amount
    if let Some(threshold) = threshold_amount {
        if expense_amount > threshold {
            let excess = expense_amount - threshold;
            return Some((
                format!("Expense amount {:.2} exceeds threshold {:.2}", expense_amount, threshold),
                excess,
            ));
        }
    }
    None
}

/// Expense Policy Compliance Engine
pub struct ExpensePolicyComplianceEngine {
    repository: Arc<dyn ExpensePolicyComplianceRepository>,
}

impl ExpensePolicyComplianceEngine {
    pub fn new(repository: Arc<dyn ExpensePolicyComplianceRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Policy Rule Management
    // ========================================================================

    /// Create a new expense policy rule
    pub async fn create_rule(
        &self,
        org_id: Uuid,
        rule_code: &str,
        name: &str,
        description: Option<&str>,
        rule_type: &str,
        expense_category: &str,
        severity: &str,
        evaluation_scope: &str,
        threshold_amount: Option<&str>,
        maximum_amount: Option<&str>,
        threshold_days: i32,
        requires_receipt: bool,
        requires_justification: bool,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        applies_to_department: Option<&str>,
        applies_to_cost_center: Option<&str>,
        created_by_id: Option<Uuid>,
    ) -> AtlasResult<ExpensePolicyRule> {
        // Validation
        if rule_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Rule code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Rule name is required".to_string()));
        }
        if !VALID_RULE_TYPES.contains(&rule_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid rule type '{}'. Must be one of: {}",
                rule_type, VALID_RULE_TYPES.join(", ")
            )));
        }
        if !VALID_EXPENSE_CATEGORIES.contains(&expense_category) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid expense category '{}'. Must be one of: {}",
                expense_category, VALID_EXPENSE_CATEGORIES.join(", ")
            )));
        }
        if !VALID_SEVERITIES.contains(&severity) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid severity '{}'. Must be one of: {}",
                severity, VALID_SEVERITIES.join(", ")
            )));
        }
        if !VALID_EVALUATION_SCOPES.contains(&evaluation_scope) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid evaluation scope '{}'. Must be one of: {}",
                evaluation_scope, VALID_EVALUATION_SCOPES.join(", ")
            )));
        }
        if threshold_days < 0 {
            return Err(AtlasError::ValidationFailed(
                "Threshold days cannot be negative".to_string(),
            ));
        }
        if let (Some(from), Some(to)) = (effective_to, effective_from) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".to_string(),
                ));
            }
        }

        info!("Creating expense policy rule {} ({}) for org {}", rule_code, name, org_id);

        self.repository.create_rule(
            org_id, rule_code, name, description, rule_type, expense_category,
            severity, evaluation_scope, threshold_amount, maximum_amount,
            threshold_days, requires_receipt, requires_justification,
            effective_from, effective_to, applies_to_department,
            applies_to_cost_center, created_by_id,
        ).await
    }

    /// Get a rule by code
    pub async fn get_rule(&self, org_id: Uuid, rule_code: &str) -> AtlasResult<Option<ExpensePolicyRule>> {
        self.repository.get_rule(org_id, rule_code).await
    }

    /// Get a rule by ID
    pub async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<ExpensePolicyRule>> {
        self.repository.get_rule_by_id(id).await
    }

    /// List rules with optional status filter
    pub async fn list_rules(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        rule_type: Option<&str>,
    ) -> AtlasResult<Vec<ExpensePolicyRule>> {
        if let Some(s) = status {
            if !VALID_RULE_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_RULE_STATUSES.join(", ")
                )));
            }
        }
        if let Some(rt) = rule_type {
            if !VALID_RULE_TYPES.contains(&rt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid rule type '{}'. Must be one of: {}",
                    rt, VALID_RULE_TYPES.join(", ")
                )));
            }
        }
        self.repository.list_rules(org_id, status, rule_type).await
    }

    /// Activate a draft rule
    pub async fn activate_rule(&self, id: Uuid) -> AtlasResult<ExpensePolicyRule> {
        let rule = self.repository.get_rule_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Rule {} not found", id)))?;

        if rule.status != "draft" && rule.status != "inactive" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot activate rule in '{}' status. Must be 'draft' or 'inactive'.",
                rule.status
            )));
        }

        info!("Activated expense policy rule {}", rule.rule_code);
        self.repository.update_rule_status(id, "active").await
    }

    /// Deactivate a rule
    pub async fn deactivate_rule(&self, id: Uuid) -> AtlasResult<ExpensePolicyRule> {
        let rule = self.repository.get_rule_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Rule {} not found", id)))?;

        if rule.status != "active" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot deactivate rule in '{}' status. Must be 'active'.",
                rule.status
            )));
        }

        info!("Deactivated expense policy rule {}", rule.rule_code);
        self.repository.update_rule_status(id, "inactive").await
    }

    /// Delete a rule (only if draft or inactive)
    pub async fn delete_rule(&self, org_id: Uuid, rule_code: &str) -> AtlasResult<()> {
        let rule = self.repository.get_rule(org_id, rule_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Rule {} not found", rule_code)))?;

        if rule.status == "active" {
            return Err(AtlasError::WorkflowError(
                "Cannot delete rule that is in 'active' status".to_string(),
            ));
        }

        info!("Deleted expense policy rule {}", rule_code);
        self.repository.delete_rule(org_id, rule_code).await
    }

    // ========================================================================
    // Compliance Audit Management
    // ========================================================================

    /// Create a compliance audit for an expense report
    pub async fn create_audit(
        &self,
        org_id: Uuid,
        report_id: Uuid,
        report_number: Option<&str>,
        employee_id: Option<Uuid>,
        employee_name: Option<&str>,
        department_id: Option<Uuid>,
        audit_trigger: &str,
        audit_date: chrono::NaiveDate,
    ) -> AtlasResult<ExpenseComplianceAudit> {
        if !VALID_AUDIT_TRIGGERS.contains(&audit_trigger) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid audit trigger '{}'. Must be one of: {}",
                audit_trigger, VALID_AUDIT_TRIGGERS.join(", ")
            )));
        }

        let next_audit_num = self.repository.get_latest_audit_number(org_id).await? + 1;
        let audit_number = format!("ECA-{}", next_audit_num);

        info!("Creating compliance audit {} for report {} in org {}", audit_number, report_id, org_id);

        self.repository.create_audit(
            org_id, &audit_number, report_id, report_number,
            employee_id, employee_name, department_id,
            audit_date, audit_trigger,
        ).await
    }

    /// Get an audit by ID
    pub async fn get_audit(&self, id: Uuid) -> AtlasResult<Option<ExpenseComplianceAudit>> {
        self.repository.get_audit(id).await
    }

    /// Get an audit by audit number
    pub async fn get_audit_by_number(&self, org_id: Uuid, audit_number: &str) -> AtlasResult<Option<ExpenseComplianceAudit>> {
        self.repository.get_audit_by_number(org_id, audit_number).await
    }

    /// List audits with optional filters
    pub async fn list_audits(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        risk_level: Option<&str>,
    ) -> AtlasResult<Vec<ExpenseComplianceAudit>> {
        if let Some(s) = status {
            if !VALID_AUDIT_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}",
                    s, VALID_AUDIT_STATUSES.join(", ")
                )));
            }
        }
        if let Some(r) = risk_level {
            if !VALID_RISK_LEVELS.contains(&r) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid risk level '{}'. Must be one of: {}",
                    r, VALID_RISK_LEVELS.join(", ")
                )));
            }
        }
        self.repository.list_audits(org_id, status, risk_level).await
    }

    /// Evaluate compliance for an audit against active policy rules
    pub async fn evaluate_compliance(&self, audit_id: Uuid) -> AtlasResult<ExpenseComplianceAudit> {
        let audit = self.repository.get_audit(audit_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Audit {} not found", audit_id)))?;

        if audit.status != "pending" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot evaluate audit in '{}' status. Must be 'pending'.",
                audit.status
            )));
        }

        // Get all active rules for the org
        let rules = self.repository.list_rules(audit.org_id, Some("active"), None).await?;

        if rules.is_empty() {
            // No active rules, mark as completed with perfect score
            return self.repository.update_audit_results(
                audit_id, 0, 0, 0, 0, "100.00", "low",
                "0.00", "0.00", false, false,
            ).await;
        }

        // Simulate evaluation: for each rule, check if it applies and create violations
        // In a full implementation, this would evaluate against actual expense report lines
        let mut violations_count = 0i32;
        let mut warnings_count = 0i32;
        let mut blocks_count = 0i32;
        let mut total_flagged = 0.0_f64;
        let total_approved = 10000.0_f64; // simulated baseline

        // For each active rule that matches, we evaluate a simulated scenario
        let simulated_expense_amount = 250.0_f64;

        for rule in &rules {
            let applies = match rule.expense_category.as_str() {
                "all" => true,
                _ => true, // In production, match against actual line categories
            };

            if !applies {
                continue;
            }

            // Evaluate based on rule type
            let mut violated = false;
            let mut violation_desc = String::new();
            let mut excess = 0.0_f64;

            match rule.rule_type.as_str() {
                "amount_limit" => {
                    let threshold = rule.threshold_amount.as_ref()
                        .and_then(|s| s.parse::<f64>().ok());
                    let maximum = rule.maximum_amount.as_ref()
                        .and_then(|s| s.parse::<f64>().ok());
                    if let Some((desc, exc)) = evaluate_amount_limit(simulated_expense_amount, threshold, maximum) {
                        violated = true;
                        violation_desc = desc;
                        excess = exc;
                    }
                }
                "receipt_required"
                    if rule.requires_receipt && simulated_expense_amount > 75.0 => {
                        violated = true;
                        violation_desc = format!("Receipt required for expenses over 75.00, got {:.2}", simulated_expense_amount);
                    }
                "daily_limit" | "category_limit" => {
                    if let Some(max_str) = &rule.maximum_amount {
                        if let Ok(max) = max_str.parse::<f64>() {
                            if simulated_expense_amount > max {
                                violated = true;
                                excess = simulated_expense_amount - max;
                                violation_desc = format!("Daily expense {:.2} exceeds limit {:.2}", simulated_expense_amount, max);
                            }
                        }
                    }
                }
                _ => {
                    // Other rule types evaluated differently in production
                }
            }

            if violated {
                let severity = match rule.severity.as_str() {
                    "block" => { blocks_count += 1; "block" }
                    "violation" => { violations_count += 1; "violation" }
                    _ => { warnings_count += 1; "warning" }
                };

                total_flagged += if excess > 0.0 { excess } else { simulated_expense_amount };

                let excess_str = if excess > 0.0 { Some(format!("{:.2}", excess)) } else { None };

                self.repository.create_violation(
                    audit.org_id, audit_id, audit.report_id,
                    None, // report_line_id
                    Some(rule.id), &rule.rule_code, Some(&rule.name),
                    &rule.rule_type, severity,
                    Some(&violation_desc),
                    Some(&format!("{:.2}", simulated_expense_amount)),
                    rule.threshold_amount.as_deref(),
                    excess_str.as_deref(),
                ).await?;
            }
        }

        let total_lines = 10i32; // Simulated line count
        let score = calculate_compliance_score(total_lines, violations_count, warnings_count, blocks_count);
        let risk_level = determine_risk_level(score);
        let requires_manager = blocks_count > 0 || score < 60.0;
        let requires_finance = blocks_count > 2 || score < 40.0;

        let updated_audit = self.repository.update_audit_results(
            audit_id,
            total_lines,
            violations_count,
            warnings_count,
            blocks_count,
            &format!("{:.2}", score),
            risk_level,
            &format!("{:.2}", total_flagged),
            &format!("{:.2}", total_approved - total_flagged),
            requires_manager,
            requires_finance,
        ).await?;

        info!(
            "Evaluated compliance audit {}: score={:.2}, risk={}, violations={}, warnings={}, blocks={}",
            audit_id, score, risk_level, violations_count, warnings_count, blocks_count
        );

        Ok(updated_audit)
    }

    /// Complete an audit review
    pub async fn complete_audit_review(
        &self,
        id: Uuid,
        reviewed_by: Option<Uuid>,
        review_notes: Option<&str>,
    ) -> AtlasResult<ExpenseComplianceAudit> {
        let audit = self.repository.get_audit(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Audit {} not found", id)))?;

        if audit.status != "pending" && audit.status != "in_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot complete review for audit in '{}' status. Must be 'pending' or 'in_review'.",
                audit.status
            )));
        }

        info!("Completed review for compliance audit {}", audit.audit_number);
        self.repository.update_audit_review(id, "completed", reviewed_by, review_notes).await
    }

    /// Escalate an audit
    pub async fn escalate_audit(
        &self,
        id: Uuid,
        reviewed_by: Option<Uuid>,
        review_notes: Option<&str>,
    ) -> AtlasResult<ExpenseComplianceAudit> {
        let audit = self.repository.get_audit(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Audit {} not found", id)))?;

        if audit.status != "pending" && audit.status != "in_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot escalate audit in '{}' status.", audit.status
            )));
        }

        info!("Escalated compliance audit {}", audit.audit_number);
        self.repository.update_audit_review(id, "escalated", reviewed_by, review_notes).await
    }

    // ========================================================================
    // Violation Management
    // ========================================================================

    /// List violations for an audit
    pub async fn list_violations(&self, audit_id: Uuid) -> AtlasResult<Vec<ExpenseComplianceViolation>> {
        self.repository.list_violations(audit_id).await
    }

    /// Resolve a violation
    pub async fn resolve_violation(
        &self,
        id: Uuid,
        resolution_status: &str,
        justification: Option<&str>,
        resolved_by: Option<Uuid>,
    ) -> AtlasResult<ExpenseComplianceViolation> {
        let violation = self.repository.get_violation_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Violation {} not found", id)))?;

        if violation.resolution_status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot resolve violation in '{}' status. Must be 'open'.",
                violation.resolution_status
            )));
        }

        if !VALID_RESOLUTION_STATUSES.contains(&resolution_status) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid resolution status '{}'. Must be one of: {}",
                resolution_status, VALID_RESOLUTION_STATUSES.join(", ")
            )));
        }

        info!("Resolved violation {} as '{}'", id, resolution_status);
        self.repository.update_violation_resolution(
            id, resolution_status, justification, resolved_by,
            Some(chrono::Utc::now().date_naive()),
        ).await
    }

    /// List open violations for an organization
    pub async fn list_open_violations(&self, org_id: Uuid) -> AtlasResult<Vec<ExpenseComplianceViolation>> {
        self.repository.list_open_violations(org_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get expense compliance dashboard summary
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<ExpenseComplianceDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_rule_types() {
        assert!(VALID_RULE_TYPES.contains(&"amount_limit"));
        assert!(VALID_RULE_TYPES.contains(&"daily_limit"));
        assert!(VALID_RULE_TYPES.contains(&"category_limit"));
        assert!(VALID_RULE_TYPES.contains(&"receipt_required"));
        assert!(VALID_RULE_TYPES.contains(&"time_restriction"));
        assert!(VALID_RULE_TYPES.contains(&"duplicate_check"));
        assert!(VALID_RULE_TYPES.contains(&"approval_required"));
        assert!(VALID_RULE_TYPES.contains(&"per_diem_override"));
    }

    #[test]
    fn test_valid_severities() {
        assert!(VALID_SEVERITIES.contains(&"warning"));
        assert!(VALID_SEVERITIES.contains(&"violation"));
        assert!(VALID_SEVERITIES.contains(&"block"));
    }

    #[test]
    fn test_valid_evaluation_scopes() {
        assert!(VALID_EVALUATION_SCOPES.contains(&"per_line"));
        assert!(VALID_EVALUATION_SCOPES.contains(&"per_day"));
        assert!(VALID_EVALUATION_SCOPES.contains(&"per_report"));
        assert!(VALID_EVALUATION_SCOPES.contains(&"per_trip"));
    }

    #[test]
    fn test_valid_expense_categories() {
        assert!(VALID_EXPENSE_CATEGORIES.contains(&"airfare"));
        assert!(VALID_EXPENSE_CATEGORIES.contains(&"hotel"));
        assert!(VALID_EXPENSE_CATEGORIES.contains(&"meals"));
        assert!(VALID_EXPENSE_CATEGORIES.contains(&"all"));
    }

    #[test]
    fn test_calculate_compliance_score_perfect() {
        let score = calculate_compliance_score(10, 0, 0, 0);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_calculate_compliance_score_with_violations() {
        let score = calculate_compliance_score(10, 2, 0, 0);
        assert_eq!(score, 80.0); // 100 - (2 * 10) = 80
    }

    #[test]
    fn test_calculate_compliance_score_with_warnings() {
        let score = calculate_compliance_score(10, 0, 5, 0);
        assert_eq!(score, 85.0); // 100 - (5 * 3) = 85
    }

    #[test]
    fn test_calculate_compliance_score_with_blocks() {
        let score = calculate_compliance_score(10, 0, 0, 2);
        assert_eq!(score, 70.0); // 100 - (2 * 15) = 70
    }

    #[test]
    fn test_calculate_compliance_score_mixed() {
        let score = calculate_compliance_score(10, 1, 2, 1);
        // 100 - 10 - 6 - 15 = 69
        assert_eq!(score, 69.0);
    }

    #[test]
    fn test_calculate_compliance_score_floor() {
        let score = calculate_compliance_score(10, 10, 20, 5);
        // 100 - 100 - 60 - 75 = -135, clamped to 0
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_calculate_compliance_score_zero_lines() {
        let score = calculate_compliance_score(0, 0, 0, 0);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_determine_risk_level_low() {
        assert_eq!(determine_risk_level(90.0), "low");
        assert_eq!(determine_risk_level(80.0), "low");
    }

    #[test]
    fn test_determine_risk_level_medium() {
        assert_eq!(determine_risk_level(70.0), "medium");
        assert_eq!(determine_risk_level(60.0), "medium");
    }

    #[test]
    fn test_determine_risk_level_high() {
        assert_eq!(determine_risk_level(50.0), "high");
        assert_eq!(determine_risk_level(40.0), "high");
    }

    #[test]
    fn test_determine_risk_level_critical() {
        assert_eq!(determine_risk_level(30.0), "critical");
        assert_eq!(determine_risk_level(0.0), "critical");
    }

    #[test]
    fn test_evaluate_amount_limit_within_threshold() {
        let result = evaluate_amount_limit(50.0, Some(100.0), None);
        assert!(result.is_none());
    }

    #[test]
    fn test_evaluate_amount_limit_exceeds_threshold() {
        let result = evaluate_amount_limit(150.0, Some(100.0), None);
        assert!(result.is_some());
        let (desc, excess) = result.unwrap();
        assert!(desc.contains("exceeds"));
        assert!((excess - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_evaluate_amount_limit_exceeds_maximum() {
        let result = evaluate_amount_limit(300.0, Some(100.0), Some(200.0));
        assert!(result.is_some());
        let (desc, excess) = result.unwrap();
        assert!(desc.contains("exceeds maximum"));
        assert!((excess - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_evaluate_amount_limit_no_thresholds() {
        let result = evaluate_amount_limit(500.0, None, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_valid_audit_triggers() {
        assert!(VALID_AUDIT_TRIGGERS.contains(&"automatic"));
        assert!(VALID_AUDIT_TRIGGERS.contains(&"random_sample"));
        assert!(VALID_AUDIT_TRIGGERS.contains(&"high_amount"));
        assert!(VALID_AUDIT_TRIGGERS.contains(&"policy_violation"));
        assert!(VALID_AUDIT_TRIGGERS.contains(&"manual"));
    }

    #[test]
    fn test_valid_resolution_statuses() {
        assert!(VALID_RESOLUTION_STATUSES.contains(&"open"));
        assert!(VALID_RESOLUTION_STATUSES.contains(&"justified"));
        assert!(VALID_RESOLUTION_STATUSES.contains(&"adjusted"));
        assert!(VALID_RESOLUTION_STATUSES.contains(&"upheld"));
        assert!(VALID_RESOLUTION_STATUSES.contains(&"escalated"));
    }
}
