//! Advanced Financial Controls Engine
//!
//! Manages continuous monitoring rules, violation detection, and
//! automated alert management for financial transactions.
//!
//! Oracle Fusion Cloud ERP equivalent: Financials > Advanced Controls

use atlas_shared::{
    ControlMonitorRule, ControlViolation, FinancialControlsDashboardSummary,
    AtlasError, AtlasResult,
};
use super::FinancialControlsRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid categories for control rules
const VALID_CATEGORIES: &[&str] = &[
    "transaction", "access", "master_data", "period_close", "master_record",
];

/// Valid risk levels
const VALID_RISK_LEVELS: &[&str] = &[
    "critical", "high", "medium", "low",
];

/// Valid control types
const VALID_CONTROL_TYPES: &[&str] = &[
    "threshold", "pattern", "frequency", "segregation", "approval", "custom",
];

/// Valid check schedules
const VALID_CHECK_SCHEDULES: &[&str] = &[
    "realtime", "daily", "weekly", "monthly",
];

/// Valid violation statuses
const VALID_VIOLATION_STATUSES: &[&str] = &[
    "open", "under_review", "resolved", "false_positive", "escalated", "waived",
];

/// Valid action types
const VALID_ACTION_TYPES: &[&str] = &[
    "alert", "block", "escalate", "review",
];

/// Advanced Financial Controls Engine
pub struct FinancialControlsEngine {
    repository: Arc<dyn FinancialControlsRepository>,
}

impl FinancialControlsEngine {
    pub fn new(repository: Arc<dyn FinancialControlsRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Control Monitor Rules
    // ========================================================================

    /// Create a new control monitor rule
    pub async fn create_rule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        category: &str,
        risk_level: &str,
        control_type: &str,
        conditions: serde_json::Value,
        threshold_value: Option<&str>,
        target_entity: &str,
        target_fields: serde_json::Value,
        actions: serde_json::Value,
        auto_resolve: bool,
        check_schedule: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<ControlMonitorRule> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Rule code and name are required".to_string(),
            ));
        }
        if !VALID_CATEGORIES.contains(&category) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid category '{}'. Must be one of: {}", category, VALID_CATEGORIES.join(", ")
            )));
        }
        if !VALID_RISK_LEVELS.contains(&risk_level) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid risk_level '{}'. Must be one of: {}", risk_level, VALID_RISK_LEVELS.join(", ")
            )));
        }
        if !VALID_CONTROL_TYPES.contains(&control_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid control_type '{}'. Must be one of: {}", control_type, VALID_CONTROL_TYPES.join(", ")
            )));
        }
        if !VALID_CHECK_SCHEDULES.contains(&check_schedule) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid check_schedule '{}'. Must be one of: {}", check_schedule, VALID_CHECK_SCHEDULES.join(", ")
            )));
        }
        if target_entity.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Target entity is required".to_string(),
            ));
        }

        // Validate actions
        if let Some(arr) = actions.as_array() {
            for action in arr {
                if let Some(action_str) = action.as_str() {
                    if !VALID_ACTION_TYPES.contains(&action_str) {
                        return Err(AtlasError::ValidationFailed(format!(
                            "Invalid action '{}'. Must be one of: {}", action_str, VALID_ACTION_TYPES.join(", ")
                        )));
                    }
                }
            }
        }

        // Validate threshold for threshold-based controls
        if control_type == "threshold" && threshold_value.is_none() {
            return Err(AtlasError::ValidationFailed(
                "Threshold value is required for threshold-based controls".to_string(),
            ));
        }

        // Check uniqueness
        if self.repository.get_rule(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(
                format!("Control rule code '{}' already exists", code)
            ));
        }

        info!("Creating control rule {} ({}/{}) for org {}", code, category, risk_level, org_id);

        self.repository.create_rule(
            org_id, code, name, description, category, risk_level, control_type,
            conditions, threshold_value, target_entity, target_fields,
            actions, auto_resolve, check_schedule, effective_from, effective_to,
            created_by,
        ).await
    }

    /// Get a rule by code
    pub async fn get_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<ControlMonitorRule>> {
        self.repository.get_rule(org_id, code).await
    }

    /// List rules with optional filters
    pub async fn list_rules(
        &self,
        org_id: Uuid,
        category: Option<&str>,
        risk_level: Option<&str>,
    ) -> AtlasResult<Vec<ControlMonitorRule>> {
        if let Some(c) = category {
            if !VALID_CATEGORIES.contains(&c) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid category '{}'. Must be one of: {}", c, VALID_CATEGORIES.join(", ")
                )));
            }
        }
        if let Some(r) = risk_level {
            if !VALID_RISK_LEVELS.contains(&r) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid risk_level '{}'. Must be one of: {}", r, VALID_RISK_LEVELS.join(", ")
                )));
            }
        }
        self.repository.list_rules(org_id, category, risk_level).await
    }

    /// Delete a rule
    pub async fn delete_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.get_rule(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Control rule '{}' not found", code)
            ))?;

        self.repository.delete_rule(org_id, code).await
    }

    // ========================================================================
    // Violations
    // ========================================================================

    /// Detect violations by evaluating rules against transaction data.
    /// This is the main continuous monitoring entry point.
    pub async fn detect_violations(
        &self,
        org_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        transaction_data: &serde_json::Value,
    ) -> AtlasResult<Vec<ControlViolation>> {
        let rules = self.repository.list_active_rules(org_id, None).await?;
        let mut violations = Vec::new();

        for rule in &rules {
            // Check if the rule applies to this entity type
            if rule.target_entity != entity_type {
                continue;
            }

            let violated = match rule.control_type.as_str() {
                "threshold" => self.evaluate_threshold_rule(rule, transaction_data),
                "pattern" => self.evaluate_pattern_rule(rule, transaction_data),
                "frequency" => self.evaluate_frequency_rule(rule, transaction_data),
                "segregation" => self.evaluate_segregation_rule(rule, transaction_data),
                "approval" => self.evaluate_approval_rule(rule, transaction_data),
                _ => false,
            };

            if violated {
                let violation = self.create_violation_from_rule(
                    org_id, rule, entity_type, Some(entity_id), transaction_data,
                ).await?;
                violations.push(violation);
            }
        }

        Ok(violations)
    }

    /// Create a violation from a rule match
    async fn create_violation_from_rule(
        &self,
        org_id: Uuid,
        rule: &ControlMonitorRule,
        entity_type: &str,
        entity_id: Option<Uuid>,
        transaction_data: &serde_json::Value,
    ) -> AtlasResult<ControlViolation> {
        let violation_number = format!("VIO-{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        let description = format!(
            "Control rule '{}' ({}) violated for {} {}",
            rule.name, rule.code, entity_type,
            entity_id.map(|id| id.to_string()).unwrap_or_default()
        );

        let findings = serde_json::json!({
            "rule_code": rule.code,
            "rule_name": rule.name,
            "control_type": rule.control_type,
            "threshold_value": rule.threshold_value,
            "transaction_data": transaction_data,
            "conditions": rule.conditions,
        });

        info!("Creating violation {} for rule {} ({})", violation_number, rule.code, rule.risk_level);

        let violation = self.repository.create_violation(
            org_id, rule.id, Some(&rule.code), Some(&rule.name),
            &violation_number, entity_type, entity_id,
            &description, findings, &rule.risk_level, "open",
            serde_json::json!({}),
        ).await?;

        // Update rule stats
        let new_total = rule.total_violations + 1;
        self.repository.update_rule_stats(
            rule.id, new_total, rule.total_resolved, Some(chrono::Utc::now()),
        ).await?;

        Ok(violation)
    }

    /// Evaluate a threshold-based control rule
    fn evaluate_threshold_rule(&self, rule: &ControlMonitorRule, data: &serde_json::Value) -> bool {
        let threshold: f64 = rule.threshold_value
            .as_ref()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        // Check target fields against threshold
        if let Some(fields) = rule.target_fields.as_array() {
            for field in fields {
                if let Some(field_name) = field.as_str() {
                    if let Some(value) = data.get(field_name) {
                        if let Some(num) = value.as_f64() {
                            // Check if the value exceeds the threshold
                            if let Some(comparison) = rule.conditions.get("comparison").and_then(|v| v.as_str()) {
                                match comparison {
                                    "greater_than" if num > threshold => return true,
                                    "less_than" if num < threshold => return true,
                                    "equals" if (num - threshold).abs() < 0.01 => return true,
                                    "not_equals" if (num - threshold).abs() >= 0.01 => return true,
                                    _ => {}
                                }
                            } else {
                                // Default: greater than
                                if num > threshold {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Evaluate a pattern-based control rule
    fn evaluate_pattern_rule(&self, rule: &ControlMonitorRule, data: &serde_json::Value) -> bool {
        // Check conditions against the transaction data
        if let Some(conditions) = rule.conditions.as_object() {
            for (field, expected) in conditions {
                if let Some(actual) = data.get(field) {
                    match expected {
                        serde_json::Value::String(s)
                            if actual.as_str().unwrap_or("") != s.as_str() => {
                                return false;
                            }
                        serde_json::Value::Bool(b)
                            if actual.as_bool().unwrap_or(false) != *b => {
                                return false;
                            }
                        _ => {}
                    }
                } else {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Evaluate a frequency-based control rule
    fn evaluate_frequency_rule(&self, rule: &ControlMonitorRule, data: &serde_json::Value) -> bool {
        // For frequency checks, look at a count field in the data
        if let Some(count_field) = rule.conditions.get("count_field").and_then(|v| v.as_str()) {
            if let Some(count) = data.get(count_field).and_then(|v| v.as_i64()) {
                let threshold: i64 = rule.threshold_value
                    .as_ref()
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(0);
                return count > threshold;
            }
        }
        false
    }

    /// Evaluate a segregation-of-duties control rule
    fn evaluate_segregation_rule(&self, rule: &ControlMonitorRule, data: &serde_json::Value) -> bool {
        // Check if same user appears in multiple conflicting fields
        if let Some(conflicting_fields) = rule.conditions.get("conflicting_fields").and_then(|v| v.as_array()) {
            let user_ids: Vec<Option<String>> = conflicting_fields.iter()
                .filter_map(|f| f.as_str())
                .map(|field| data.get(field).and_then(|v| v.as_str()).map(|s| s.to_string()))
                .collect();

            // Check for any duplicates (same user in multiple conflicting roles)
            let non_none: Vec<&str> = user_ids.iter()
                .filter_map(|opt| opt.as_deref())
                .collect();

            for (i, id) in non_none.iter().enumerate() {
                for (j, other) in non_none.iter().enumerate() {
                    if i != j && *id == *other {
                        return true; // Segregation violation detected
                    }
                }
            }
        }
        false
    }

    /// Evaluate an approval-based control rule
    fn evaluate_approval_rule(&self, rule: &ControlMonitorRule, data: &serde_json::Value) -> bool {
        // Check if the approval chain has issues
        let requires_approval = rule.conditions.get("requires_approval")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if requires_approval {
            let has_approval = data.get("approved_by")
                .and_then(|v| v.as_str())
                .is_some();

            if !has_approval {
                return true; // Missing required approval
            }
        }
        false
    }

    // ========================================================================
    // Violation Management
    // ========================================================================

    /// Get a violation by ID
    pub async fn get_violation(&self, id: Uuid) -> AtlasResult<Option<ControlViolation>> {
        self.repository.get_violation(id).await
    }

    /// List violations with optional filters
    pub async fn list_violations(
        &self,
        org_id: Uuid,
        status: Option<&str>,
        risk_level: Option<&str>,
        rule_id: Option<Uuid>,
        assigned_to: Option<Uuid>,
    ) -> AtlasResult<Vec<ControlViolation>> {
        if let Some(s) = status {
            if !VALID_VIOLATION_STATUSES.contains(&s) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid status '{}'. Must be one of: {}", s, VALID_VIOLATION_STATUSES.join(", ")
                )));
            }
        }
        if let Some(r) = risk_level {
            if !VALID_RISK_LEVELS.contains(&r) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid risk_level '{}'. Must be one of: {}", r, VALID_RISK_LEVELS.join(", ")
                )));
            }
        }
        self.repository.list_violations(org_id, status, risk_level, rule_id, assigned_to).await
    }

    /// Assign a violation to a reviewer
    pub async fn assign_violation(
        &self,
        violation_id: Uuid,
        assigned_to: Uuid,
        assigned_to_name: &str,
    ) -> AtlasResult<ControlViolation> {
        let violation = self.repository.get_violation(violation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Violation {} not found", violation_id)
            ))?;

        if violation.status != "open" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot assign violation in '{}' status. Must be 'open'.", violation.status
            )));
        }

        info!("Assigning violation {} to {}", violation.violation_number, assigned_to_name);
        self.repository.update_violation_status(
            violation_id, "under_review", Some(assigned_to),
            Some(assigned_to_name), None, None,
        ).await
    }

    /// Resolve a violation
    pub async fn resolve_violation(
        &self,
        violation_id: Uuid,
        resolution_notes: &str,
        resolved_by: Uuid,
    ) -> AtlasResult<ControlViolation> {
        let violation = self.repository.get_violation(violation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Violation {} not found", violation_id)
            ))?;

        if violation.status != "open" && violation.status != "under_review" && violation.status != "escalated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot resolve violation in '{}' status.", violation.status
            )));
        }
        if resolution_notes.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Resolution notes are required".to_string(),
            ));
        }

        info!("Resolving violation {} by {}", violation.violation_number, resolved_by);
        self.repository.update_violation_status(
            violation_id, "resolved", None, None,
            Some(resolution_notes), Some(resolved_by),
        ).await
    }

    /// Mark a violation as a false positive
    pub async fn mark_false_positive(
        &self,
        violation_id: Uuid,
        resolution_notes: &str,
        resolved_by: Uuid,
    ) -> AtlasResult<ControlViolation> {
        let violation = self.repository.get_violation(violation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Violation {} not found", violation_id)
            ))?;

        if violation.status != "open" && violation.status != "under_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot mark violation as false positive in '{}' status.", violation.status
            )));
        }

        info!("Marking violation {} as false positive", violation.violation_number);
        self.repository.update_violation_status(
            violation_id, "false_positive", None, None,
            Some(resolution_notes), Some(resolved_by),
        ).await
    }

    /// Escalate a violation
    pub async fn escalate_violation(
        &self,
        violation_id: Uuid,
        escalated_to: Uuid,
    ) -> AtlasResult<ControlViolation> {
        let violation = self.repository.get_violation(violation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Violation {} not found", violation_id)
            ))?;

        if violation.status != "open" && violation.status != "under_review" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot escalate violation in '{}' status.", violation.status
            )));
        }

        info!("Escalating violation {} to {}", violation.violation_number, escalated_to);
        self.repository.escalate_violation(violation_id, Some(escalated_to), Some(chrono::Utc::now())).await
    }

    /// Waive a violation
    pub async fn waive_violation(
        &self,
        violation_id: Uuid,
        resolution_notes: &str,
        resolved_by: Uuid,
    ) -> AtlasResult<ControlViolation> {
        let violation = self.repository.get_violation(violation_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(
                format!("Violation {} not found", violation_id)
            ))?;

        if violation.status != "open" && violation.status != "under_review" && violation.status != "escalated" {
            return Err(AtlasError::WorkflowError(format!(
                "Cannot waive violation in '{}' status.", violation.status
            )));
        }

        info!("Waiving violation {}", violation.violation_number);
        self.repository.update_violation_status(
            violation_id, "waived", None, None,
            Some(resolution_notes), Some(resolved_by),
        ).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the financial controls dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<FinancialControlsDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_categories() {
        assert!(VALID_CATEGORIES.contains(&"transaction"));
        assert!(VALID_CATEGORIES.contains(&"access"));
        assert!(VALID_CATEGORIES.contains(&"master_data"));
        assert!(VALID_CATEGORIES.contains(&"period_close"));
        assert!(VALID_CATEGORIES.contains(&"master_record"));
        assert_eq!(VALID_CATEGORIES.len(), 5);
    }

    #[test]
    fn test_valid_risk_levels() {
        assert!(VALID_RISK_LEVELS.contains(&"critical"));
        assert!(VALID_RISK_LEVELS.contains(&"high"));
        assert!(VALID_RISK_LEVELS.contains(&"medium"));
        assert!(VALID_RISK_LEVELS.contains(&"low"));
        assert_eq!(VALID_RISK_LEVELS.len(), 4);
    }

    #[test]
    fn test_valid_control_types() {
        assert!(VALID_CONTROL_TYPES.contains(&"threshold"));
        assert!(VALID_CONTROL_TYPES.contains(&"pattern"));
        assert!(VALID_CONTROL_TYPES.contains(&"frequency"));
        assert!(VALID_CONTROL_TYPES.contains(&"segregation"));
        assert!(VALID_CONTROL_TYPES.contains(&"approval"));
        assert!(VALID_CONTROL_TYPES.contains(&"custom"));
        assert_eq!(VALID_CONTROL_TYPES.len(), 6);
    }

    #[test]
    fn test_valid_check_schedules() {
        assert!(VALID_CHECK_SCHEDULES.contains(&"realtime"));
        assert!(VALID_CHECK_SCHEDULES.contains(&"daily"));
        assert!(VALID_CHECK_SCHEDULES.contains(&"weekly"));
        assert!(VALID_CHECK_SCHEDULES.contains(&"monthly"));
    }

    #[test]
    fn test_valid_violation_statuses() {
        assert!(VALID_VIOLATION_STATUSES.contains(&"open"));
        assert!(VALID_VIOLATION_STATUSES.contains(&"under_review"));
        assert!(VALID_VIOLATION_STATUSES.contains(&"resolved"));
        assert!(VALID_VIOLATION_STATUSES.contains(&"false_positive"));
        assert!(VALID_VIOLATION_STATUSES.contains(&"escalated"));
        assert!(VALID_VIOLATION_STATUSES.contains(&"waived"));
    }

    #[test]
    fn test_valid_action_types() {
        assert!(VALID_ACTION_TYPES.contains(&"alert"));
        assert!(VALID_ACTION_TYPES.contains(&"block"));
        assert!(VALID_ACTION_TYPES.contains(&"escalate"));
        assert!(VALID_ACTION_TYPES.contains(&"review"));
    }

    #[test]
    fn test_evaluate_threshold_greater_than() {
        let engine = create_test_engine();

        let rule = ControlMonitorRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: "TEST".to_string(),
            name: "Test".to_string(),
            description: None,
            category: "transaction".to_string(),
            risk_level: "high".to_string(),
            control_type: "threshold".to_string(),
            conditions: serde_json::json!({"comparison": "greater_than"}),
            threshold_value: Some("10000".to_string()),
            target_entity: "journal_entry".to_string(),
            target_fields: serde_json::json!(["amount"]),
            actions: serde_json::json!(["alert"]),
            auto_resolve: false,
            check_schedule: "realtime".to_string(),
            is_active: true,
            effective_from: None,
            effective_to: None,
            last_check_at: None,
            last_violation_at: None,
            total_violations: 0,
            total_resolved: 0,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Over threshold - should violate
        assert!(engine.evaluate_threshold_rule(&rule, &serde_json::json!({"amount": 15000.0})));

        // Under threshold - should not violate
        assert!(!engine.evaluate_threshold_rule(&rule, &serde_json::json!({"amount": 5000.0})));

        // At threshold - should not violate (strictly greater)
        assert!(!engine.evaluate_threshold_rule(&rule, &serde_json::json!({"amount": 10000.0})));
    }

    #[test]
    fn test_evaluate_threshold_less_than() {
        let engine = create_test_engine();

        let rule = ControlMonitorRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: "TEST".to_string(),
            name: "Test".to_string(),
            description: None,
            category: "transaction".to_string(),
            risk_level: "high".to_string(),
            control_type: "threshold".to_string(),
            conditions: serde_json::json!({"comparison": "less_than"}),
            threshold_value: Some("100".to_string()),
            target_entity: "payment".to_string(),
            target_fields: serde_json::json!(["amount"]),
            actions: serde_json::json!(["alert"]),
            auto_resolve: false,
            check_schedule: "realtime".to_string(),
            is_active: true,
            effective_from: None,
            effective_to: None,
            last_check_at: None,
            last_violation_at: None,
            total_violations: 0,
            total_resolved: 0,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Under threshold - should violate
        assert!(engine.evaluate_threshold_rule(&rule, &serde_json::json!({"amount": 50.0})));

        // Over threshold - should not violate
        assert!(!engine.evaluate_threshold_rule(&rule, &serde_json::json!({"amount": 200.0})));
    }

    #[test]
    fn test_evaluate_pattern_rule() {
        let engine = create_test_engine();

        let rule = ControlMonitorRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: "PATTERN".to_string(),
            name: "Pattern Test".to_string(),
            description: None,
            category: "transaction".to_string(),
            risk_level: "medium".to_string(),
            control_type: "pattern".to_string(),
            conditions: serde_json::json!({"country": "HIGH_RISK", "is_new_vendor": true}),
            threshold_value: None,
            target_entity: "invoice".to_string(),
            target_fields: serde_json::json!([]),
            actions: serde_json::json!(["review"]),
            auto_resolve: false,
            check_schedule: "realtime".to_string(),
            is_active: true,
            effective_from: None,
            effective_to: None,
            last_check_at: None,
            last_violation_at: None,
            total_violations: 0,
            total_resolved: 0,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // All conditions match
        assert!(engine.evaluate_pattern_rule(&rule, &serde_json::json!({
            "country": "HIGH_RISK", "is_new_vendor": true, "amount": 5000.0
        })));

        // Missing condition
        assert!(!engine.evaluate_pattern_rule(&rule, &serde_json::json!({
            "country": "HIGH_RISK", "amount": 5000.0
        })));

        // Non-matching condition
        assert!(!engine.evaluate_pattern_rule(&rule, &serde_json::json!({
            "country": "SAFE", "is_new_vendor": true
        })));
    }

    #[test]
    fn test_evaluate_segregation_rule() {
        let engine = create_test_engine();

        let rule = ControlMonitorRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: "SOD".to_string(),
            name: "Segregation Test".to_string(),
            description: None,
            category: "access".to_string(),
            risk_level: "critical".to_string(),
            control_type: "segregation".to_string(),
            conditions: serde_json::json!({"conflicting_fields": ["preparer_id", "approver_id"]}),
            threshold_value: None,
            target_entity: "journal_entry".to_string(),
            target_fields: serde_json::json!([]),
            actions: serde_json::json!(["block", "escalate"]),
            auto_resolve: false,
            check_schedule: "realtime".to_string(),
            is_active: true,
            effective_from: None,
            effective_to: None,
            last_check_at: None,
            last_violation_at: None,
            total_violations: 0,
            total_resolved: 0,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Same user in both roles - violation
        assert!(engine.evaluate_segregation_rule(&rule, &serde_json::json!({
            "preparer_id": "user-123", "approver_id": "user-123"
        })));

        // Different users - no violation
        assert!(!engine.evaluate_segregation_rule(&rule, &serde_json::json!({
            "preparer_id": "user-123", "approver_id": "user-456"
        })));
    }

    #[test]
    fn test_evaluate_approval_rule() {
        let engine = create_test_engine();

        let rule = ControlMonitorRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: "APPR".to_string(),
            name: "Approval Test".to_string(),
            description: None,
            category: "transaction".to_string(),
            risk_level: "high".to_string(),
            control_type: "approval".to_string(),
            conditions: serde_json::json!({"requires_approval": true}),
            threshold_value: None,
            target_entity: "payment".to_string(),
            target_fields: serde_json::json!([]),
            actions: serde_json::json!(["block"]),
            auto_resolve: false,
            check_schedule: "realtime".to_string(),
            is_active: true,
            effective_from: None,
            effective_to: None,
            last_check_at: None,
            last_violation_at: None,
            total_violations: 0,
            total_resolved: 0,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // No approval - violation
        assert!(engine.evaluate_approval_rule(&rule, &serde_json::json!({
            "amount": 50000.0
        })));

        // Has approval - no violation
        assert!(!engine.evaluate_approval_rule(&rule, &serde_json::json!({
            "amount": 50000.0, "approved_by": "manager-001"
        })));
    }

    #[test]
    fn test_evaluate_frequency_rule() {
        let engine = create_test_engine();

        let rule = ControlMonitorRule {
            id: Uuid::new_v4(),
            organization_id: Uuid::new_v4(),
            code: "FREQ".to_string(),
            name: "Frequency Test".to_string(),
            description: None,
            category: "transaction".to_string(),
            risk_level: "medium".to_string(),
            control_type: "frequency".to_string(),
            conditions: serde_json::json!({"count_field": "transaction_count"}),
            threshold_value: Some("5".to_string()),
            target_entity: "payment".to_string(),
            target_fields: serde_json::json!([]),
            actions: serde_json::json!(["alert"]),
            auto_resolve: false,
            check_schedule: "daily".to_string(),
            is_active: true,
            effective_from: None,
            effective_to: None,
            last_check_at: None,
            last_violation_at: None,
            total_violations: 0,
            total_resolved: 0,
            metadata: serde_json::json!({}),
            created_by: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Over frequency threshold - violation
        assert!(engine.evaluate_frequency_rule(&rule, &serde_json::json!({
            "transaction_count": 10
        })));

        // Under threshold - no violation
        assert!(!engine.evaluate_frequency_rule(&rule, &serde_json::json!({
            "transaction_count": 3
        })));
    }

    fn create_test_engine() -> FinancialControlsEngine {
        FinancialControlsEngine::new(Arc::new(crate::mock_repos::MockFinancialControlsRepository))
    }
}
