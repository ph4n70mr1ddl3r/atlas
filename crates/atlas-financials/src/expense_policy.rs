use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpensePolicyRule {
    pub id: Uuid,
    pub org_id: Uuid,
    pub rule_code: String,
    pub rule_type: String, // amount_limit, receipt_required
    pub expense_category: String,
    pub severity: String, // warning, violation, block
    pub maximum_amount: Decimal,
    pub requires_receipt: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluationResult {
    pub is_compliant: bool,
    pub severity: String,
    pub message: String,
}

pub struct ExpensePolicyService {
    rules: Arc<RwLock<Vec<ExpensePolicyRule>>>,
}

impl Default for ExpensePolicyService {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpensePolicyService {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn create_rule(
        &self,
        org_id: Uuid,
        code: String,
        rule_type: String,
        category: String,
        severity: String,
        max_amt: Decimal,
        receipt: bool,
    ) -> Result<ExpensePolicyRule, String> {
        let mut rules = self.rules.write().unwrap();
        if rules
            .iter()
            .any(|r| r.org_id == org_id && r.rule_code == code)
        {
            return Err("Rule already exists".to_string());
        }

        let rule = ExpensePolicyRule {
            id: Uuid::new_v4(),
            org_id,
            rule_code: code,
            rule_type,
            expense_category: category,
            severity,
            maximum_amount: max_amt,
            requires_receipt: receipt,
        };

        rules.push(rule.clone());
        Ok(rule)
    }

    pub fn evaluate_line(
        &self,
        org_id: Uuid,
        category: &str,
        amount: Decimal,
        has_receipt: bool,
    ) -> Vec<PolicyEvaluationResult> {
        let rules = self.rules.read().unwrap();
        let mut results = Vec::new();

        for rule in rules.iter().filter(|r| {
            r.org_id == org_id && (r.expense_category == "all" || r.expense_category == category)
        }) {
            match rule.rule_type.as_str() {
                "amount_limit" if amount > rule.maximum_amount => {
                    results.push(PolicyEvaluationResult {
                        is_compliant: false,
                        severity: rule.severity.clone(),
                        message: format!(
                            "Amount {} exceeds limit of {}",
                            amount, rule.maximum_amount
                        ),
                    });
                }
                "receipt_required" if rule.requires_receipt && !has_receipt => {
                    results.push(PolicyEvaluationResult {
                        is_compliant: false,
                        severity: rule.severity.clone(),
                        message: "Receipt is required for this expense".to_string(),
                    });
                }
                "amount_limit" | "receipt_required" => {}
                _ => {}
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_evaluate_amount_limit() {
        let service = ExpensePolicyService::new();
        let org_id = Uuid::new_v4();

        service
            .create_rule(
                org_id,
                "MEAL_LIMIT".to_string(),
                "amount_limit".to_string(),
                "meals".to_string(),
                "violation".to_string(),
                dec!(50),
                false,
            )
            .unwrap();

        // Compliant
        let r1 = service.evaluate_line(org_id, "meals", dec!(40), false);
        assert!(r1.is_empty());

        // Violation
        let r2 = service.evaluate_line(org_id, "meals", dec!(60), false);
        assert_eq!(r2.len(), 1);
        assert_eq!(r2[0].severity, "violation");
    }

    #[test]
    fn test_evaluate_receipt_required() {
        let service = ExpensePolicyService::new();
        let org_id = Uuid::new_v4();

        service
            .create_rule(
                org_id,
                "RCPT_REQ".to_string(),
                "receipt_required".to_string(),
                "all".to_string(),
                "block".to_string(),
                dec!(0),
                true,
            )
            .unwrap();

        // No receipt -> Block
        let r = service.evaluate_line(org_id, "hotel", dec!(200), false);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].severity, "block");

        // Receipt -> OK
        let r2 = service.evaluate_line(org_id, "hotel", dec!(200), true);
        assert!(r2.is_empty());
    }
}
