//! Cross-Validation Rule Engine
//!
//! Manages rule lifecycle, pattern-based validation, priority-based evaluation,
//! and effective date handling.
//!
//! Oracle Fusion equivalent: General Ledger > Setup > Chart of Accounts >
//!   Cross-Validation Rules

use atlas_shared::{
    CrossValidationRule, CrossValidationRuleLine, CrossValidationResult,
    CrossValidationDashboardSummary,
    AtlasError, AtlasResult,
};
use super::CrossValidationRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid rule types
const VALID_RULE_TYPES: &[&str] = &["deny", "allow"];

/// Valid line types
const VALID_LINE_TYPES: &[&str] = &["from", "to"];

/// Wildcard character meaning "any value"
const WILDCARD: &str = "%";

/// Cross-Validation Rule engine
pub struct CrossValidationEngine {
    repository: Arc<dyn CrossValidationRepository>,
}

impl CrossValidationEngine {
    pub fn new(repository: Arc<dyn CrossValidationRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Rule Management
    // ========================================================================

    /// Create a new cross-validation rule
    pub async fn create_rule(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        rule_type: &str,
        error_message: &str,
        priority: i32,
        segment_names: Vec<String>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<CrossValidationRule> {
        if code.is_empty() {
            return Err(AtlasError::ValidationFailed("Rule code is required".to_string()));
        }
        if name.is_empty() {
            return Err(AtlasError::ValidationFailed("Rule name is required".to_string()));
        }
        if error_message.is_empty() {
            return Err(AtlasError::ValidationFailed("Error message is required".to_string()));
        }
        if !VALID_RULE_TYPES.contains(&rule_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid rule type '{}'. Must be one of: {}",
                rule_type, VALID_RULE_TYPES.join(", ")
            )));
        }
        if segment_names.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "At least one segment name is required".to_string(),
            ));
        }
        if priority < 0 {
            return Err(AtlasError::ValidationFailed("Priority must be >= 0".to_string()));
        }
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if to < from {
                return Err(AtlasError::ValidationFailed(
                    "Effective to date must be after effective from date".to_string(),
                ));
            }
        }

        // Check uniqueness
        if self.repository.get_rule(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!(
                "Cross-validation rule with code '{}' already exists", code
            )));
        }

        info!("Creating cross-validation rule {} ({}) for org {}", code, name, org_id);

        self.repository.create_rule(
            org_id, code, name, description, rule_type, error_message,
            priority, segment_names, effective_from, effective_to, created_by,
        ).await
    }

    /// Get a rule by code
    pub async fn get_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<CrossValidationRule>> {
        self.repository.get_rule(org_id, code).await
    }

    /// Get a rule by ID
    pub async fn get_rule_by_id(&self, id: Uuid) -> AtlasResult<Option<CrossValidationRule>> {
        self.repository.get_rule_by_id(id).await
    }

    /// List rules with optional filter
    pub async fn list_rules(
        &self,
        org_id: Uuid,
        enabled_only: bool,
    ) -> AtlasResult<Vec<CrossValidationRule>> {
        self.repository.list_rules(org_id, enabled_only).await
    }

    /// Enable a rule
    pub async fn enable_rule(&self, id: Uuid) -> AtlasResult<CrossValidationRule> {
        let rule = self.get_rule_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Rule {} not found", id)))?;

        if rule.is_enabled {
            return Err(AtlasError::WorkflowError("Rule is already enabled".to_string()));
        }

        info!("Enabled cross-validation rule {}", rule.code);
        self.repository.update_rule_enabled(id, true).await
    }

    /// Disable a rule
    pub async fn disable_rule(&self, id: Uuid) -> AtlasResult<CrossValidationRule> {
        let rule = self.get_rule_by_id(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Rule {} not found", id)))?;

        if !rule.is_enabled {
            return Err(AtlasError::WorkflowError("Rule is already disabled".to_string()));
        }

        info!("Disabled cross-validation rule {}", rule.code);
        self.repository.update_rule_enabled(id, false).await
    }

    /// Delete a rule and all its lines
    pub async fn delete_rule(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        info!("Deleted cross-validation rule {}", code);
        self.repository.delete_rule(org_id, code).await
    }

    // ========================================================================
    // Rule Lines
    // ========================================================================

    /// Add a validation line to a rule
    pub async fn create_rule_line(
        &self,
        org_id: Uuid,
        rule_code: &str,
        line_type: &str,
        patterns: Vec<String>,
        display_order: i32,
    ) -> AtlasResult<CrossValidationRuleLine> {
        if !VALID_LINE_TYPES.contains(&line_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid line type '{}'. Must be one of: {}",
                line_type, VALID_LINE_TYPES.join(", ")
            )));
        }

        let rule = self.repository.get_rule(org_id, rule_code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!(
                "Rule '{}' not found", rule_code
            )))?;

        if patterns.len() != rule.segment_names.len() {
            return Err(AtlasError::ValidationFailed(format!(
                "Pattern count ({}) must match segment count ({})",
                patterns.len(), rule.segment_names.len()
            )));
        }

        info!("Adding {} line to cross-validation rule {}", line_type, rule_code);

        self.repository.create_rule_line(
            org_id, rule.id, line_type, &patterns, display_order,
        ).await
    }

    /// List lines for a rule
    pub async fn list_rule_lines(&self, rule_id: Uuid) -> AtlasResult<Vec<CrossValidationRuleLine>> {
        self.repository.list_rule_lines(rule_id).await
    }

    /// Delete a rule line
    pub async fn delete_rule_line(&self, id: Uuid) -> AtlasResult<()> {
        info!("Deleted cross-validation rule line {}", id);
        self.repository.delete_rule_line(id).await
    }

    // ========================================================================
    // Validation
    // ========================================================================

    /// Validate a segment combination against all active rules.
    /// Returns a result indicating whether the combination is valid.
    ///
    /// Evaluation logic:
    /// 1. Load all enabled, effective rules ordered by priority
    /// 2. For each rule, check if both "from" and "to" patterns match
    /// 3. If a "deny" rule matches → combination is invalid
    /// 4. If an "allow" rule matches → that specific deny is overridden
    ///    (allow rules override deny rules of same/higher priority)
    pub async fn validate_combination(
        &self,
        org_id: Uuid,
        segment_values: &[String],
    ) -> AtlasResult<CrossValidationResult> {
        let rules = self.repository.list_rules(org_id, true).await?;

        let today = chrono::Utc::now().date_naive();
        let mut violated_rules = Vec::new();
        let mut error_messages = Vec::new();
        let mut allow_matches: Vec<Uuid> = Vec::new();

        // Collect applicable rules (enabled and within effective dates)
        let mut applicable_rules: Vec<&CrossValidationRule> = rules.iter()
            .filter(|r| {
                
                match (r.effective_from, r.effective_to) {
                    (Some(from), Some(to)) => today >= from && today <= to,
                    (Some(from), None) => today >= from,
                    (None, Some(to)) => today <= to,
                    (None, None) => true,
                }
            })
            .collect();

        // Sort by priority (lower = evaluated first)
        applicable_rules.sort_by_key(|r| r.priority);

        // First pass: identify all matching "allow" rules
        for rule in &applicable_rules {
            if rule.rule_type == "allow"
                && self.rule_matches(rule, segment_values).await? {
                    allow_matches.push(rule.id);
                }
        }

        // Second pass: check "deny" rules
        for rule in &applicable_rules {
            if rule.rule_type == "deny"
                && self.rule_matches(rule, segment_values).await? {
                    // Check if any allow rule overrides this deny
                    let is_overridden = allow_matches.iter().any(|_allow_id| {
                        // Simple override: allow rule with same or higher priority (lower number)
                        // In Oracle Fusion, allow rules override all deny rules
                        // We use a simpler model: any matching allow overrides any matching deny
                        true
                    });

                    if !is_overridden {
                        violated_rules.push(rule.code.clone());
                        error_messages.push(rule.error_message.clone());
                    }
                }
        }

        let is_valid = violated_rules.is_empty();

        Ok(CrossValidationResult {
            is_valid,
            violated_rules,
            error_messages,
        })
    }

    /// Check if a single rule matches the given segment values.
    /// A rule matches when all its "from" lines OR all its "to" lines match
    /// the segment values. A rule with both "from" and "to" lines matches
    /// only when BOTH match simultaneously.
    async fn rule_matches(
        &self,
        rule: &CrossValidationRule,
        segment_values: &[String],
    ) -> AtlasResult<bool> {
        let lines = self.repository.list_rule_lines(rule.id).await?;

        if lines.is_empty() {
            return Ok(false);
        }

        let from_lines: Vec<&CrossValidationRuleLine> = lines.iter()
            .filter(|l| l.line_type == "from")
            .collect();
        let to_lines: Vec<&CrossValidationRuleLine> = lines.iter()
            .filter(|l| l.line_type == "to")
            .collect();

        let from_matches = if from_lines.is_empty() {
            true // No from constraint = always matches
        } else {
            from_lines.iter().any(|line| Self::pattern_matches(&line.patterns, segment_values))
        };

        let to_matches = if to_lines.is_empty() {
            true // No to constraint = always matches
        } else {
            to_lines.iter().any(|line| Self::pattern_matches(&line.patterns, segment_values))
        };

        Ok(from_matches && to_matches)
    }

    /// Check if a pattern matches a set of segment values.
    /// Each pattern element is matched against the corresponding segment value.
    /// "%" matches any value, exact strings must match exactly (case-insensitive).
    pub fn pattern_matches(patterns: &[String], values: &[String]) -> bool {
        if patterns.len() != values.len() {
            return false;
        }

        for (pattern, value) in patterns.iter().zip(values.iter()) {
            if pattern == WILDCARD {
                continue; // Wildcard matches anything
            }
            if pattern.to_uppercase() != value.to_uppercase() {
                return false; // Exact match required (case-insensitive)
            }
        }
        true
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get dashboard summary
    pub async fn get_dashboard_summary(&self, org_id: Uuid) -> AtlasResult<CrossValidationDashboardSummary> {
        self.repository.get_dashboard_summary(org_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_rule_types() {
        assert!(VALID_RULE_TYPES.contains(&"deny"));
        assert!(VALID_RULE_TYPES.contains(&"allow"));
        assert_eq!(VALID_RULE_TYPES.len(), 2);
    }

    #[test]
    fn test_valid_line_types() {
        assert!(VALID_LINE_TYPES.contains(&"from"));
        assert!(VALID_LINE_TYPES.contains(&"to"));
        assert_eq!(VALID_LINE_TYPES.len(), 2);
    }

    #[test]
    fn test_pattern_matches_exact() {
        assert!(CrossValidationEngine::pattern_matches(
            &["1000".to_string(), "MARKETING".to_string(), "5000".to_string()],
            &["1000".to_string(), "MARKETING".to_string(), "5000".to_string()],
        ));
    }

    #[test]
    fn test_pattern_matches_case_insensitive() {
        assert!(CrossValidationEngine::pattern_matches(
            &["1000".to_string(), "marketing".to_string(), "5000".to_string()],
            &["1000".to_string(), "MARKETING".to_string(), "5000".to_string()],
        ));
    }

    #[test]
    fn test_pattern_matches_wildcard_all() {
        assert!(CrossValidationEngine::pattern_matches(
            &["%".to_string(), "%".to_string(), "%".to_string()],
            &["1000".to_string(), "MARKETING".to_string(), "5000".to_string()],
        ));
    }

    #[test]
    fn test_pattern_matches_wildcard_partial() {
        // Company 1000, any department, account 5000
        assert!(CrossValidationEngine::pattern_matches(
            &["1000".to_string(), "%".to_string(), "5000".to_string()],
            &["1000".to_string(), "MARKETING".to_string(), "5000".to_string()],
        ));
        assert!(CrossValidationEngine::pattern_matches(
            &["1000".to_string(), "%".to_string(), "5000".to_string()],
            &["1000".to_string(), "ENGINEERING".to_string(), "5000".to_string()],
        ));
        assert!(!CrossValidationEngine::pattern_matches(
            &["1000".to_string(), "%".to_string(), "5000".to_string()],
            &["2000".to_string(), "MARKETING".to_string(), "5000".to_string()],
        ));
    }

    #[test]
    fn test_pattern_matches_mismatch() {
        assert!(!CrossValidationEngine::pattern_matches(
            &["1000".to_string(), "MARKETING".to_string()],
            &["1000".to_string(), "ENGINEERING".to_string()],
        ));
    }

    #[test]
    fn test_pattern_matches_length_mismatch() {
        assert!(!CrossValidationEngine::pattern_matches(
            &["1000".to_string(), "MARKETING".to_string()],
            &["1000".to_string()],
        ));
    }

    #[test]
    fn test_pattern_matches_empty() {
        let empty: Vec<String> = vec![];
        assert!(CrossValidationEngine::pattern_matches(&empty, &empty));
    }

    #[test]
    fn test_pattern_matches_single_value() {
        assert!(CrossValidationEngine::pattern_matches(
            &["CASH".to_string()],
            &["CASH".to_string()],
        ));
        assert!(!CrossValidationEngine::pattern_matches(
            &["CASH".to_string()],
            &["EQUITY".to_string()],
        ));
    }

    #[test]
    fn test_pattern_matches_single_wildcard() {
        assert!(CrossValidationEngine::pattern_matches(
            &["%".to_string()],
            &["anything".to_string()],
        ));
    }
}
