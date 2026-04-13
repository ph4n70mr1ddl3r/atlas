//! Validation Engine Implementation

use atlas_shared::{EntityDefinition, FieldDefinition, FieldType, ValidationRule};
use atlas_shared::errors::AtlasResult;
use super::{ValidationContext, ValidationResult, ValidationError};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::debug;
use regex::Regex;

/// Validation engine for declarative validation
pub struct ValidationEngine {
    custom_validators: HashMap<String, CustomValidator>,
    /// Cache compiled regex patterns for better performance
    regex_cache: Arc<RwLock<HashMap<String, Regex>>>,
}

type CustomValidator = Arc<dyn Fn(&serde_json::Value, &str) -> Option<String> + Send + Sync>;

impl ValidationEngine {
    pub fn new() -> Self {
        Self {
            custom_validators: HashMap::new(),
            regex_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get a compiled regex from cache, compiling if needed
    fn get_cached_regex(&self, pattern: &str) -> Option<Regex> {
        // Try to get from cache first (sync lock)
        {
            let cache = self.regex_cache.read();
            if let Some(regex) = cache.get(pattern) {
                return Some(regex.clone());
            }
        }
        
        // Compile and cache
        if let Ok(regex) = Regex::new(pattern) {
            let mut cache = self.regex_cache.write();
            cache.insert(pattern.to_string(), regex.clone());
            return Some(regex);
        }
        
        None
    }
    
    /// Clear the regex cache
    pub fn clear_regex_cache(&self) {
        let mut cache = self.regex_cache.write();
        cache.clear();
    }
    
    /// Register a custom validator
    pub fn register_validator(&mut self, name: &str, validator: CustomValidator) {
        self.custom_validators.insert(name.to_string(), validator);
    }
    
    /// Validate data against an entity definition
    pub fn validate(&self, entity: &EntityDefinition, data: &serde_json::Value, ctx: Option<&ValidationContext>) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Validate required fields
        for field in &entity.fields {
            if field.is_required {
                self.validate_required(&mut result, &field, data);
            }
            
            // Validate field-specific rules
            for rule in &field.validations {
                self.validate_rule(&mut result, &field, &rule, data);
            }
            
            // Type-specific validation
            self.validate_field_type(&mut result, &field, data);
        }
        
        result
    }
    
    /// Validate a single field
    pub fn validate_field(&self, field: &FieldDefinition, value: &serde_json::Value) -> ValidationResult {
        let mut result = ValidationResult::new();
        let field_name = &field.name;
        let data = serde_json::json!({ field_name: value });
        
        if field.is_required {
            self.validate_required(&mut result, field, &data);
        }
        
        self.validate_field_type(&mut result, field, &data);
        
        for rule in &field.validations {
            self.validate_rule(&mut result, field, rule, &data);
        }
        
        result
    }
    
    fn validate_required(&self, result: &mut ValidationResult, field: &FieldDefinition, data: &serde_json::Value) {
        if let Some(value) = data.get(&field.name) {
            if value.is_null() {
                result.add_error(
                    &field.name,
                    "required",
                    &format!("{} is required", field.label)
                );
            } else if let Some(s) = value.as_str() {
                if s.trim().is_empty() {
                    result.add_error(
                        &field.name,
                        "required",
                        &format!("{} cannot be empty", field.label)
                    );
                }
            }
        } else {
            result.add_error(
                &field.name,
                "required",
                &format!("{} is required", field.label)
            );
        }
    }
    
    fn validate_field_type(&self, result: &mut ValidationResult, field: &FieldDefinition, data: &serde_json::Value) {
        let value = match data.get(&field.name) {
            Some(v) => v,
            None => return, // Skip if field not present
        };
        
        if value.is_null() {
            return;
        }
        
        match &field.field_type {
            FieldType::Integer { min, max } => {
                if let Some(n) = value.as_i64() {
                    if let Some(min_val) = min {
                        if n < *min_val {
                            result.add_error(&field.name, "min", &format!("{} must be at least {}", field.label, min_val));
                        }
                    }
                    if let Some(max_val) = max {
                        if n > *max_val {
                            result.add_error(&field.name, "max", &format!("{} must be at most {}", field.label, max_val));
                        }
                    }
                } else if !value.is_null() {
                    result.add_error(&field.name, "type", &format!("{} must be an integer", field.label));
                }
            }
            
            FieldType::Decimal { precision, scale } => {
                if let Some(n) = value.as_f64() {
                    // Check scale (decimal places)
                    let scale_val = 10_f64.powi(*scale as i32);
                    let int_part = (n * scale_val).round() / scale_val;
                    let actual_scale = if n != 0.0 {
                        let diff = ((n - int_part) * scale_val).abs();
                        if diff > 0.0 { diff.log10().ceil() as u8 } else { 0 }
                    } else {
                        0
                    };
                    
                    if actual_scale > *scale {
                        result.add_error(&field.name, "scale", &format!("{} can have at most {} decimal places", field.label, scale));
                    }
                } else if !value.is_null() && !value.is_string() {
                    result.add_error(&field.name, "type", &format!("{} must be a number", field.label));
                }
            }
            
            FieldType::Enum { values } => {
                if let Some(s) = value.as_str() {
                    if !values.contains(&s.to_string()) {
                        result.add_error(
                            &field.name, 
                            "enum", 
                            &format!("{} must be one of: {}", field.label, values.join(", "))
                        );
                    }
                } else {
                    result.add_error(&field.name, "type", &format!("{} must be a string", field.label));
                }
            }
            
            FieldType::Email => {
                if let Some(s) = value.as_str() {
                    if !s.contains('@') || !s.contains('.') {
                        result.add_error(&field.name, "email", &format!("{} must be a valid email", field.label));
                    }
                }
            }
            
            FieldType::Url => {
                if let Some(s) = value.as_str() {
                    if !s.starts_with("http://") && !s.starts_with("https://") {
                        result.add_error(&field.name, "url", &format!("{} must be a valid URL", field.label));
                    }
                }
            }
            
            _ => {} // Other types validated elsewhere
        }
    }
    
    fn validate_rule(&self, result: &mut ValidationResult, field: &FieldDefinition, rule: &ValidationRule, data: &serde_json::Value) {
        let value = match data.get(&field.name) {
            Some(v) => v,
            None => return,
        };
        
        match rule {
            ValidationRule::Required => {
                if value.is_null() || (value.is_string() && value.as_str().unwrap().trim().is_empty()) {
                    result.add_error(&field.name, "required", &format!("{} is required", field.label));
                }
            }
            
            ValidationRule::MinLength { value: min } => {
                if let Some(s) = value.as_str() {
                    if s.len() < *min {
                        result.add_error(&field.name, "minLength", &format!("{} must be at least {} characters", field.label, min));
                    }
                }
            }
            
            ValidationRule::MaxLength { value: max } => {
                if let Some(s) = value.as_str() {
                    if s.len() > *max {
                        result.add_error(&field.name, "maxLength", &format!("{} must be at most {} characters", field.label, max));
                    }
                }
            }
            
            ValidationRule::Min { value: min } => {
                if let Some(n) = value.as_f64() {
                    if n < *min {
                        result.add_error(&field.name, "min", &format!("{} must be at least {}", field.label, min));
                    }
                }
            }
            
            ValidationRule::Max { value: max } => {
                if let Some(n) = value.as_f64() {
                    if n > *max {
                        result.add_error(&field.name, "max", &format!("{} must be at most {}", field.label, max));
                    }
                }
            }
            
            ValidationRule::Pattern { value: pattern } => {
                if let Some(s) = value.as_str() {
                    // Use cached regex for better performance
                    if let Some(re) = self.get_cached_regex(pattern) {
                        if !re.is_match(s) {
                            result.add_error(&field.name, "pattern", &format!("{} does not match required format", field.label));
                        }
                    } else {
                        result.add_error(&field.name, "pattern", &format!("Invalid regex pattern: {}", pattern));
                    }
                }
            }
            
            ValidationRule::Email => {
                if let Some(s) = value.as_str() {
                    if !s.contains('@') || !s.contains('.') {
                        result.add_error(&field.name, "email", &format!("{} must be a valid email", field.label));
                    }
                }
            }
            
            ValidationRule::Url => {
                if let Some(s) = value.as_str() {
                    if !s.starts_with("http://") && !s.starts_with("https://") {
                        result.add_error(&field.name, "url", &format!("{} must be a valid URL", field.label));
                    }
                }
            }
            
            ValidationRule::Custom { expression, message } => {
                debug!("Custom validation: {} with expression {}", field.name, expression);
                // Custom validators would be looked up and called here
            }
        }
    }
    
    /// Cross-field validation
    pub fn validate_cross_field(&self, data: &serde_json::Value, rules: &[(String, String, String)]) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        for (field1, operator, field2) in rules {
            let value1 = data.get(field1);
            let value2 = data.get(field2);
            
            let valid = match operator.as_str() {
                "=" | "==" | "equals" => value1 == value2,
                "!=" | "not_equals" => value1 != value2,
                ">" => {
                    if let (Some(v1), Some(v2)) = (value1.and_then(|v| v.as_f64()), value2.and_then(|v| v.as_f64())) {
                        v1 > v2
                    } else { true }
                }
                "<" => {
                    if let (Some(v1), Some(v2)) = (value1.and_then(|v| v.as_f64()), value2.and_then(|v| v.as_f64())) {
                        v1 < v2
                    } else { true }
                }
                ">=" => {
                    if let (Some(v1), Some(v2)) = (value1.and_then(|v| v.as_f64()), value2.and_then(|v| v.as_f64())) {
                        v1 >= v2
                    } else { true }
                }
                "<=" => {
                    if let (Some(v1), Some(v2)) = (value1.and_then(|v| v.as_f64()), value2.and_then(|v| v.as_f64())) {
                        v1 <= v2
                    } else { true }
                }
                _ => true,
            };
            
            if !valid {
                result.add_error(
                    field1,
                    "cross_field",
                    &format!("{} {} {} (cross-field validation)", field1, operator, field2)
                );
            }
        }
        
        result
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_shared::{FieldDefinition, FieldType};
    
    fn create_test_entity() -> EntityDefinition {
        EntityDefinition {
            id: None,
            name: "test".to_string(),
            label: "Test".to_string(),
            plural_label: "Tests".to_string(),
            table_name: None,
            description: None,
            fields: vec![
                FieldDefinition::new("name", "Name", FieldType::String { max_length: Some(100), pattern: None }),
                FieldDefinition::new("amount", "Amount", FieldType::Decimal { precision: 10, scale: 2 }),
                FieldDefinition::new("email", "Email", FieldType::Email),
                FieldDefinition::new("status", "Status", FieldType::Enum { values: vec!["draft".to_string(), "active".to_string()] }),
            ],
            indexes: vec![],
            workflow: None,
            security: None,
            is_audit_enabled: true,
            is_soft_delete: true,
            icon: None,
            color: None,
            metadata: serde_json::Value::Null,
        }
    }
    
    #[test]
    fn test_required_field() {
        let engine = ValidationEngine::new();
        let entity = create_test_entity();
        
        // Find the name field and make it required
        let mut name_field = entity.fields[0].clone();
        name_field.is_required = true;
        
        let data = serde_json::json!({});
        let result = engine.validate_field(&name_field, &serde_json::Value::Null);
        
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "required"));
    }
    
    #[test]
    fn test_email_validation() {
        let engine = ValidationEngine::new();
        let entity = create_test_entity();
        
        let email_field = &entity.fields[2];
        
        // Valid email
        let result = engine.validate_field(email_field, &serde_json::json!("test@example.com"));
        assert!(result.valid);
        
        // Invalid email
        let result = engine.validate_field(email_field, &serde_json::json!("not-an-email"));
        assert!(!result.valid);
    }
    
    #[test]
    fn test_enum_validation() {
        let engine = ValidationEngine::new();
        let entity = create_test_entity();
        
        let status_field = &entity.fields[3];
        
        // Valid enum value
        let result = engine.validate_field(status_field, &serde_json::json!("draft"));
        assert!(result.valid);
        
        // Invalid enum value
        let result = engine.validate_field(status_field, &serde_json::json!("invalid"));
        assert!(!result.valid);
    }
    
    #[test]
    fn test_cross_field_validation() {
        let engine = ValidationEngine::new();
        
        let data = serde_json::json!({
            "start_date": "2024-01-01",
            "end_date": "2024-01-15"
        });
        
        let rules = vec![
            ("start_date".to_string(), "<".to_string(), "end_date".to_string()),
        ];
        
        let result = engine.validate_cross_field(&data, &rules);
        assert!(result.valid);
    }
}
