//! Oracle Fusion Financial Feature: Journal Approval Routing
//! Manages approval requirements and routing rules for General Ledger journal batches.

pub struct JournalApprovalService;

#[derive(Debug, PartialEq, Clone)]
pub struct JournalApprovalRule {
    pub rule_name: String,
    pub source_name: Option<String>,
    pub min_amount: f64,
    pub max_amount: Option<f64>,
    pub is_auto_approved: bool,
    pub approver_role: Option<String>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct JournalBatch {
    pub batch_id: String,
    pub source: String,
    pub total_accounted_cr: f64,
}

#[derive(Debug, PartialEq)]
pub struct ApprovalRoutingResult {
    pub status: String, // 'AUTO_APPROVED', 'ROUTED_FOR_APPROVAL', 'NO_RULE_FOUND'
    pub assigned_role: Option<String>,
    pub rule_applied: Option<String>,
}

impl JournalApprovalService {
    /// Evaluates a journal batch against active approval rules to determine if it can be
    /// auto-approved or if it needs to be routed to a specific role/manager.
    #[must_use]
    pub fn determine_routing(
        batch: &JournalBatch,
        rules: &[JournalApprovalRule],
    ) -> ApprovalRoutingResult {
        // Find all rules that match the source and amount thresholds
        let matching_rules: Vec<&JournalApprovalRule> = rules
            .iter()
            .filter(|r| {
                let source_match = r.source_name.as_deref().unwrap_or(&batch.source) == batch.source;
                let min_match = batch.total_accounted_cr >= r.min_amount;
                let max_match = match r.max_amount {
                    Some(max) => batch.total_accounted_cr <= max,
                    None => true,
                };
                source_match && min_match && max_match
            })
            .collect();

        if matching_rules.is_empty() {
            // In Oracle Fusion, if no rule matches and approval is required, it often goes to a default error queue or requires admin intervention.
            return ApprovalRoutingResult {
                status: "NO_RULE_FOUND".to_string(),
                assigned_role: None,
                rule_applied: None,
            };
        }

        // We pick the most specific rule (usually Oracle has priority logic, we'll take the first match for simplicity)
        let rule = matching_rules[0];

        if rule.is_auto_approved {
            return ApprovalRoutingResult {
                status: "AUTO_APPROVED".to_string(),
                assigned_role: None,
                rule_applied: Some(rule.rule_name.clone()),
            };
        }

        ApprovalRoutingResult {
            status: "ROUTED_FOR_APPROVAL".to_string(),
            assigned_role: rule.approver_role.clone(),
            rule_applied: Some(rule.rule_name.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_approved_small_amount() {
        let rules = vec![
            JournalApprovalRule {
                rule_name: "Auto-Approve Under 1000".to_string(),
                source_name: None, // Applies to all
                min_amount: 0.0,
                max_amount: Some(1000.0),
                is_auto_approved: true,
                approver_role: None,
            }
        ];

        let batch = JournalBatch {
            batch_id: "JB-100".to_string(),
            source: "Spreadsheet".to_string(),
            total_accounted_cr: 500.0,
        };

        let result = JournalApprovalService::determine_routing(&batch, &rules);

        assert_eq!(result.status, "AUTO_APPROVED");
        assert_eq!(result.assigned_role, None);
        assert_eq!(result.rule_applied, Some("Auto-Approve Under 1000".to_string()));
    }

    #[test]
    fn test_routed_for_approval_manager() {
        let rules = vec![
            JournalApprovalRule {
                rule_name: "Manager Approval for 1K-5K".to_string(),
                source_name: None,
                min_amount: 1000.01,
                max_amount: Some(5000.0),
                is_auto_approved: false,
                approver_role: Some("FINANCE_MANAGER".to_string()),
            }
        ];

        let batch = JournalBatch {
            batch_id: "JB-101".to_string(),
            source: "Manual".to_string(),
            total_accounted_cr: 2500.0,
        };

        let result = JournalApprovalService::determine_routing(&batch, &rules);

        assert_eq!(result.status, "ROUTED_FOR_APPROVAL");
        assert_eq!(result.assigned_role, Some("FINANCE_MANAGER".to_string()));
        assert_eq!(result.rule_applied, Some("Manager Approval for 1K-5K".to_string()));
    }

    #[test]
    fn test_routed_for_approval_controller_unbounded() {
        let rules = vec![
            JournalApprovalRule {
                rule_name: "Controller Approval over 5K".to_string(),
                source_name: None,
                min_amount: 5000.01,
                max_amount: None, // Unbounded
                is_auto_approved: false,
                approver_role: Some("FINANCIAL_CONTROLLER".to_string()),
            }
        ];

        let batch = JournalBatch {
            batch_id: "JB-102".to_string(),
            source: "Manual".to_string(),
            total_accounted_cr: 10_000_000.0,
        };

        let result = JournalApprovalService::determine_routing(&batch, &rules);

        assert_eq!(result.status, "ROUTED_FOR_APPROVAL");
        assert_eq!(result.assigned_role, Some("FINANCIAL_CONTROLLER".to_string()));
        assert_eq!(result.rule_applied, Some("Controller Approval over 5K".to_string()));
    }

    #[test]
    fn test_source_specific_rule() {
        let rules = vec![
            JournalApprovalRule {
                rule_name: "Payables Integration Auto-Approve".to_string(),
                source_name: Some("Payables".to_string()),
                min_amount: 0.0,
                max_amount: None,
                is_auto_approved: true,
                approver_role: None,
            }
        ];

        let batch = JournalBatch {
            batch_id: "JB-103".to_string(),
            source: "Payables".to_string(),
            total_accounted_cr: 50000.0, // High amount, but source is trusted
        };

        let result = JournalApprovalService::determine_routing(&batch, &rules);

        assert_eq!(result.status, "AUTO_APPROVED");
        assert_eq!(result.rule_applied, Some("Payables Integration Auto-Approve".to_string()));
    }

    #[test]
    fn test_no_rule_found() {
        let rules = vec![
            JournalApprovalRule {
                rule_name: "Only Spreadsheets".to_string(),
                source_name: Some("Spreadsheet".to_string()),
                min_amount: 0.0,
                max_amount: Some(100.0),
                is_auto_approved: true,
                approver_role: None,
            }
        ];

        // This batch does not match the source or the amount
        let batch = JournalBatch {
            batch_id: "JB-104".to_string(),
            source: "Manual".to_string(),
            total_accounted_cr: 500.0,
        };

        let result = JournalApprovalService::determine_routing(&batch, &rules);

        assert_eq!(result.status, "NO_RULE_FOUND");
        assert_eq!(result.rule_applied, None);
    }
}
