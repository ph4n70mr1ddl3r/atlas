//! Workflow Guards
//! 
//! Guard conditions that control when transitions can execute.

use atlas_shared::GuardDefinition;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Result of guard evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardResult {
    pub passed: bool,
    pub message: Option<String>,
}

impl GuardResult {
    pub fn pass() -> Self {
        Self { passed: true, message: None }
    }
    
    pub fn fail(message: &str) -> Self {
        Self { passed: false, message: Some(message.to_string()) }
    }
}

/// Evaluates guard conditions for workflow transitions
pub struct GuardEvaluator;

impl GuardEvaluator {
    pub fn new() -> Self {
        Self
    }
    
    /// Evaluate a guard condition against record data
    pub fn evaluate(&self, guard: &GuardDefinition, record_data: &serde_json::Value) -> GuardResult {
        match guard {
            GuardDefinition::Validate { rule } => {
                self.evaluate_validation_rule(rule, record_data)
            }
            GuardDefinition::Expression { expression } => {
                self.evaluate_expression(expression, record_data)
            }
            GuardDefinition::Role { roles: _ } => {
                // Role checking is handled at the engine level
                GuardResult::pass()
            }
            GuardDefinition::Custom { handler } => {
                self.evaluate_custom_handler(handler, record_data)
            }
        }
    }
    
    fn evaluate_validation_rule(&self, rule: &str, data: &serde_json::Value) -> GuardResult {
        // Parse validation rules like "required(field_name)" or "not_empty(field_name)"
        let rule = rule.trim();
        
        if rule.starts_with("validate.required(") && rule.ends_with(')') {
            let field = extract_field_name(rule, "validate.required(");
            if let Some(value) = data.get(field) {
                if value.is_null() || value.as_str().is_some_and(|s| s.is_empty()) {
                    return GuardResult::fail(&format!("{} is required", field));
                }
            }
        }
        
        if rule.starts_with("validate.not_empty(") && rule.ends_with(')') {
            let field = extract_field_name(rule, "validate.not_empty(");
            if let Some(value) = data.get(field) {
                if value.is_null() || value.as_str().is_some_and(|s| s.is_empty()) {
                    return GuardResult::fail(&format!("{} cannot be empty", field));
                }
            }
        }
        
        if rule.starts_with("validate.greater_than(") && rule.ends_with(')') {
            // Format: validate.greater_than(field, value)
            let content = &rule["validate.greater_than(".len()..rule.len()-1];
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let field = parts[0];
                let threshold: f64 = parts[1].parse().unwrap_or(0.0);
                
                if let Some(value) = data.get(field) {
                    if let Some(num) = value.as_f64() {
                        if num <= threshold {
                            return GuardResult::fail(&format!("{} must be greater than {}", field, threshold));
                        }
                    }
                }
            }
        }
        
        if rule.starts_with("validate.equals(") && rule.ends_with(')') {
            // Format: validate.equals(field, value)
            let content = &rule["validate.equals(".len()..rule.len()-1];
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                let field = parts[0];
                let expected = parts[1].trim_matches('"');
                
                if let Some(value) = data.get(field) {
                    if let Some(actual) = value.as_str() {
                        if actual != expected {
                            return GuardResult::fail(&format!("{} must equal {}", field, expected));
                        }
                    }
                }
            }
        }
        
        GuardResult::pass()
    }
    
    fn evaluate_expression(&self, expression: &str, data: &serde_json::Value) -> GuardResult {
        // Simple expression evaluator
        // Supports: field == value, field > value, field < value, etc.
        
        let expression = expression.trim();
        
        // Check for field existence
        if let Some(paren_idx) = expression.find('(') {
            let func_name = &expression[..paren_idx];
            let args = &expression[paren_idx+1..expression.len()-1];
            
            match func_name {
                "hasValue" => {
                    let field = args.trim();
                    if let Some(value) = data.get(field) {
                        if value.is_null() {
                            return GuardResult::fail(&format!("{} must have a value", field));
                        }
                    }
                }
                "isEmpty" => {
                    let field = args.trim();
                    if let Some(value) = data.get(field) {
                        if !value.is_null() && !value.as_str().is_none_or(|s| s.is_empty()) {
                            return GuardResult::fail(&format!("{} must be empty", field));
                        }
                    }
                }
                _ => {
                    debug!("Unknown expression function: {}", func_name);
                }
            }
        }
        
        GuardResult::pass()
    }
    
    fn evaluate_custom_handler(&self, handler: &str, _data: &serde_json::Value) -> GuardResult {
        // Custom handlers would be registered and called dynamically
        // For now, we just log and pass
        debug!("Custom guard handler: {}", handler);
        GuardResult::pass()
    }
}

fn extract_field_name<'a>(rule: &'a str, prefix: &str) -> &'a str {
    rule[prefix.len()..rule.len()-1].trim()
}

impl Default for GuardEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_shared::GuardDefinition;
    
    #[test]
    fn test_required_validation() {
        let evaluator = GuardEvaluator::new();
        
        let data = serde_json::json!({
            "name": "Test",
            "description": null
        });
        
        let guard = GuardDefinition::Validate { 
            rule: "validate.required(name)".to_string() 
        };
        assert!(evaluator.evaluate(&guard, &data).passed);
        
        let guard2 = GuardDefinition::Validate { 
            rule: "validate.required(description)".to_string() 
        };
        assert!(!evaluator.evaluate(&guard2, &data).passed);
    }
    
    #[test]
    fn test_greater_than_validation() {
        let evaluator = GuardEvaluator::new();
        
        let data = serde_json::json!({
            "amount": 100.0,
            "quantity": 0
        });
        
        let guard = GuardDefinition::Validate { 
            rule: "validate.greater_than(amount, 0)".to_string() 
        };
        assert!(evaluator.evaluate(&guard, &data).passed);
        
        let guard2 = GuardDefinition::Validate { 
            rule: "validate.greater_than(quantity, 0)".to_string() 
        };
        assert!(!evaluator.evaluate(&guard2, &data).passed);
    }
    
    #[test]
    fn test_not_empty_validation() {
        let evaluator = GuardEvaluator::new();
        
        let data = serde_json::json!({
            "notes": "   ",
            "comment": ""
        });
        
        let guard = GuardDefinition::Validate { 
            rule: "validate.not_empty(notes)".to_string() 
        };
        // Note: This treats "   " as non-empty. A real implementation might trim.
        // For now, we just check is_empty on the string
        assert!(evaluator.evaluate(&guard, &data).passed);
        
        let guard2 = GuardDefinition::Validate { 
            rule: "validate.not_empty(comment)".to_string() 
        };
        assert!(!evaluator.evaluate(&guard2, &data).passed);
    }
}
