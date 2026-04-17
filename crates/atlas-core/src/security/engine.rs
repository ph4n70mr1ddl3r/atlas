//! Security Engine Implementation

use atlas_shared::{SecurityPolicy, SecurityRule};
use super::{SecurityContext, AccessDecision, FieldSecurity, FieldCheck};
use std::collections::HashMap;
use tracing::debug;

/// Tri-state result for evaluating a single security rule.
enum RuleMatch {
    /// The rule explicitly allows the action.
    Allow,
    /// The rule explicitly denies the action (with reason).
    Deny(String),
    /// The rule does not apply to this action/context.
    NotApplicable,
}

/// Security engine for access control
pub struct SecurityEngine {
    policies: HashMap<String, SecurityPolicy>,
    field_security: HashMap<String, HashMap<String, FieldSecurity>>,
}

impl SecurityEngine {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
            field_security: HashMap::new(),
        }
    }
    
    /// Load a security policy
    pub fn load_policy(&mut self, entity: &str, policy: SecurityPolicy) {
        debug!("Loading security policy for entity: {}", entity);
        self.policies.insert(entity.to_string(), policy);
    }
    
    /// Set field-level security for an entity
    pub fn set_field_security(&mut self, entity: &str, field: &str, security: FieldSecurity) {
        self.field_security
            .entry(entity.to_string())
            .or_default()
            .insert(field.to_string(), security);
    }
    
    /// Check if user can perform an action on an entity
    pub fn check_access(&self, entity: &str, action: &str, ctx: &SecurityContext, record_data: Option<&serde_json::Value>) -> AccessDecision {
        // Admin bypass
        if ctx.roles.contains(&"admin".to_string()) || ctx.roles.contains(&"system".to_string()) {
            return AccessDecision { allowed: true, reason: Some("Admin bypass".to_string()) };
        }
        
        // Get policy for entity
        if let Some(policy) = self.policies.get(entity) {
            let mut allowed = false;
            for rule in &policy.rules {
                match self.evaluate_rule(rule, action, ctx, record_data) {
                    RuleMatch::Allow => allowed = true,
                    RuleMatch::Deny(reason) => return AccessDecision { allowed: false, reason: Some(reason) },
                    RuleMatch::NotApplicable => {},
                }
            }
            if allowed {
                return AccessDecision { allowed: true, reason: Some("Allowed by policy".to_string()) };
            }
        }
        
        // Default: allow reads and workflow actions, deny destructive writes
        match action {
            "read" | "execute_action" | "list" | "history" | "transitions" => {
                AccessDecision { allowed: true, reason: Some("Default read/action allowed".to_string()) }
            }
            "create" | "update" | "delete" => AccessDecision { allowed: false, reason: Some("No write access".to_string()) },
            _ => AccessDecision { allowed: false, reason: Some(format!("Unknown action: {}", action)) },
        }
    }
    
    fn evaluate_rule(&self, rule: &SecurityRule, action: &str, ctx: &SecurityContext, record_data: Option<&serde_json::Value>) -> RuleMatch {
        match rule {
            SecurityRule::Allow { actions, condition } => {
                if !actions.contains(&action.to_string()) {
                    return RuleMatch::NotApplicable;
                }
                if let Some(cond) = condition {
                    if let Some(data) = record_data {
                        if !self.evaluate_condition(cond, ctx, data) {
                            return RuleMatch::NotApplicable;
                        }
                    }
                }
                RuleMatch::Allow
            }
            SecurityRule::Deny { actions, condition } => {
                if !actions.contains(&action.to_string()) {
                    return RuleMatch::NotApplicable;
                }
                if let Some(cond) = condition {
                    if let Some(data) = record_data {
                        if !self.evaluate_condition(cond, ctx, data) {
                            return RuleMatch::NotApplicable;
                        }
                    }
                }
                RuleMatch::Deny("Rule denies".to_string())
            }
        }
    }
    
    fn evaluate_condition(&self, condition: &str, ctx: &SecurityContext, record_data: &serde_json::Value) -> bool {
        // Simple condition evaluation
        // Format: "field == value" or "field == user.field"
        
        if let Some(op_pos) = condition.find("==") {
            let field = condition[..op_pos].trim();
            let value = condition[op_pos + 2..].trim();
            
            let record_value = record_data.get(field);
            
            // Check for user field reference
            if let Some(user_field) = value.strip_prefix("user.") {
            match user_field {
                    "organization_id" => {
                        if let (Some(rec_org), Some(ctx_org)) = (
                            record_value.and_then(|v| v.as_str()),
                            ctx.organization_id
                        ) {
                            return rec_org == ctx_org.to_string();
                        }
                    }
                    "roles" => {
                        // Check if user has any of the specified roles
                        if let Some(roles_array) = record_value {
                            if let Some(roles) = roles_array.as_array() {
                                return roles.iter().any(|r| {
                                    ctx.roles.contains(&r.as_str().unwrap_or("").to_string())
                                });
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                // Direct value comparison
                if let Some(rec_val) = record_value {
                    return rec_val == value;
                }
            }
        }
        
        true
    }
    
    /// Check field-level security
    pub fn check_field_access(&self, entity: &str, field: &str, ctx: &SecurityContext) -> FieldCheck {
        // Admin bypass
        if ctx.roles.contains(&"admin".to_string()) {
            return FieldCheck {
                field: field.to_string(),
                can_read: true,
                can_write: true,
                visible: true,
            };
        }
        
        // Get field security
        let field_sec = self.field_security
            .get(entity)
            .and_then(|m| m.get(field));
        
        if let Some(sec) = field_sec {
            FieldCheck {
                field: field.to_string(),
                can_read: sec.read_roles.is_empty() || sec.read_roles.iter().any(|r| ctx.roles.contains(r)),
                can_write: sec.write_roles.iter().any(|r| ctx.roles.contains(r)),
                visible: !sec.hidden,
            }
        } else {
            // Default: readable, not writable
            FieldCheck {
                field: field.to_string(),
                can_read: true,
                can_write: false,
                visible: true,
            }
        }
    }
    
    /// Filter record data based on field security
    pub fn filter_record(&self, entity: &str, data: &serde_json::Value, ctx: &SecurityContext) -> serde_json::Value {
        if let Some(obj) = data.as_object() {
            let mut filtered = serde_json::Map::new();
            
            for (key, value) in obj {
                let check = self.check_field_access(entity, key, ctx);
                if check.visible {
                    filtered.insert(key.clone(), value.clone());
                }
            }
            
            serde_json::Value::Object(filtered)
        } else {
            data.clone()
        }
    }
    
    /// Build SQL filter for row-level security
    pub fn build_rls_filter(&self, _entity: &str, ctx: &SecurityContext) -> Option<String> {
        if ctx.roles.contains(&"admin".to_string()) || ctx.roles.contains(&"system".to_string()) {
            return None; // No filter for admins
        }
        
        // Organization filter (parameterized by caller)
        if let Some(_org_id) = ctx.organization_id {
            return Some("organization_id = $1".to_string());
        }
        
        None
    }
}

impl Default for SecurityEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_context() -> SecurityContext {
        SecurityContext {
            user_id: Some(uuid::Uuid::new_v4()),
            organization_id: Some(uuid::Uuid::new_v4()),
            roles: vec!["user".to_string()],
            session_id: Some(uuid::Uuid::new_v4()),
        }
    }
    
    #[test]
    fn test_admin_bypass() {
        let engine = SecurityEngine::new();
        let mut ctx = create_context();
        ctx.roles = vec!["admin".to_string()];
        
        let decision = engine.check_access("any_entity", "delete", &ctx, None);
        assert!(decision.allowed);
    }
    
    #[test]
    fn test_read_allowed() {
        let engine = SecurityEngine::new();
        let ctx = create_context();
        
        let decision = engine.check_access("any_entity", "read", &ctx, None);
        assert!(decision.allowed);
    }
    
    #[test]
    fn test_write_denied_by_default() {
        let engine = SecurityEngine::new();
        let ctx = create_context();
        
        let decision = engine.check_access("any_entity", "update", &ctx, None);
        assert!(!decision.allowed);
    }
    
    #[test]
    fn test_field_security() {
        let mut engine = SecurityEngine::new();
        
        engine.set_field_security("employees", "salary", FieldSecurity {
            read_roles: vec!["hr_admin".to_string(), "payroll".to_string()],
            write_roles: vec!["payroll".to_string()],
            hidden: false,
        });
        
        // User without access
        let ctx = create_context();
        let check = engine.check_field_access("employees", "salary", &ctx);
        assert!(!check.can_read);
        assert!(!check.can_write);
        
        // User with read access
        let mut ctx = create_context();
        ctx.roles = vec!["hr_admin".to_string()];
        let check = engine.check_field_access("employees", "salary", &ctx);
        assert!(check.can_read);
        assert!(!check.can_write);
        
        // User with write access
        let mut ctx = create_context();
        ctx.roles = vec!["payroll".to_string()];
        let check = engine.check_field_access("employees", "salary", &ctx);
        assert!(check.can_read);
        assert!(check.can_write);
    }
}
