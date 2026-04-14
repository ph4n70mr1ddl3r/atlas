//! Row-Level Security (RLS)
//!
//! SQL generation for dynamic row-level security filters.

use super::SecurityContext;
use std::collections::HashMap;

/// Row-level security rule
#[derive(Debug, Clone)]
pub struct RlsRule {
    /// The condition expressed as a SQL fragment
    pub condition: String,
    /// Roles this rule applies to (empty = all roles)
    pub roles: Vec<String>,
    /// Whether this rule applies on INSERT
    pub for_insert: bool,
    /// Whether this rule applies on UPDATE
    pub for_update: bool,
    /// Whether this rule applies on DELETE
    pub for_delete: bool,
}

/// RLS filter builder
pub struct RlsFilterBuilder {
    rules: HashMap<String, Vec<RlsRule>>,
}

impl RlsFilterBuilder {
    pub fn new() -> Self {
        Self { rules: HashMap::new() }
    }

    /// Add an RLS rule for an entity
    pub fn add_rule(&mut self, entity: &str, rule: RlsRule) {
        self.rules
            .entry(entity.to_string())
            .or_insert_with(Vec::new)
            .push(rule);
    }

    /// Build the WHERE clause for an entity.
    ///
    /// Returns `Some(condition)` with `$N` placeholders.  The caller is
    /// responsible for binding the corresponding values via
    /// `bind_rls_values`.
    pub fn build_filter(&self, entity: &str, ctx: &SecurityContext) -> Option<String> {
        let rules = self.rules.get(entity)?;

        let mut conditions: Vec<String> = vec![];
        for rule in rules.iter()
            .filter(|rule| {
                rule.roles.is_empty() || rule.roles.iter().any(|r| ctx.roles.contains(r))
            })
        {
            let (cond, _n) = self.substitute_placeholders(&rule.condition, ctx, conditions.len() as i32 + 1);
            conditions.push(cond);
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }

    /// Substitute placeholders in a condition with `$N` parameter markers.
    ///
    /// Returns the substituted condition string and the next parameter index.
    fn substitute_placeholders(&self, condition: &str, ctx: &SecurityContext, start_idx: i32) -> (String, i32) {
        let mut result = condition.to_string();
        let mut idx = start_idx;

        if ctx.user_id.is_some() && result.contains("{{user_id}}") {
            result = result.replace("{{user_id}}", &format!("${}", idx));
            idx += 1;
        }

        if ctx.organization_id.is_some() && result.contains("{{organization_id}}") {
            result = result.replace("{{organization_id}}", &format!("${}", idx));
            idx += 1;
        }

        (result, idx)
    }

    /// Build INSERT check (for checking if user can insert).
    ///
    /// Same parameterized approach as `build_filter`.
    pub fn build_insert_check(&self, entity: &str, ctx: &SecurityContext) -> Option<String> {
        let rules = self.rules.get(entity)?;

        let mut conditions: Vec<String> = vec![];
        for rule in rules.iter()
            .filter(|rule| rule.for_insert)
            .filter(|rule| {
                rule.roles.is_empty() || rule.roles.iter().any(|r| ctx.roles.contains(r))
            })
        {
            let (cond, _) = self.substitute_placeholders(&rule.condition, ctx, conditions.len() as i32 + 1);
            conditions.push(cond);
        }

        if conditions.is_empty() {
            None
        } else {
            Some(conditions.join(" AND "))
        }
    }
}

/// Common RLS patterns
pub mod patterns {
    use crate::security::rls::RlsRule;

    /// Organization-based RLS
    pub fn org_filter(field: &str) -> RlsRule {
        RlsRule {
            condition: format!("{} = {{{{organization_id}}}}", field),
            roles: vec![],
            for_insert: true,
            for_update: true,
            for_delete: true,
        }
    }

    /// Owner-based RLS
    pub fn owner_filter(field: &str) -> RlsRule {
        RlsRule {
            condition: format!("{} = {{{{user_id}}}}", field),
            roles: vec![],
            for_insert: true,
            for_update: true,
            for_delete: true,
        }
    }

    /// Role-based RLS
    pub fn role_filter(condition: &str, roles: Vec<&str>) -> RlsRule {
        RlsRule {
            condition: condition.to_string(),
            roles: roles.into_iter().map(|s| s.to_string()).collect(),
            for_insert: true,
            for_update: true,
            for_delete: true,
        }
    }
}

impl Default for RlsFilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_context() -> SecurityContext {
        SecurityContext {
            user_id: Some(uuid::Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()),
            organization_id: Some(uuid::Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap()),
            roles: vec!["user".to_string()],
            session_id: None,
        }
    }

    #[test]
    fn test_org_filter() {
        let mut builder = RlsFilterBuilder::new();
        builder.add_rule("orders", patterns::org_filter("organization_id"));
        
        let ctx = create_context();
        let filter = builder.build_filter("orders", &ctx);
        
        assert!(filter.is_some());
        let f = filter.unwrap();
        // Parameterized — should contain $1, not the raw UUID
        assert!(f.contains("organization_id = $"));
        assert!(!f.contains("22222222"));
    }

    #[test]
    fn test_owner_filter() {
        let mut builder = RlsFilterBuilder::new();
        builder.add_rule("tasks", patterns::owner_filter("assigned_to"));
        
        let ctx = create_context();
        let filter = builder.build_filter("tasks", &ctx);
        
        assert!(filter.is_some());
        let f = filter.unwrap();
        // Parameterized — should contain $1, not the raw UUID
        assert!(f.contains("assigned_to = $"));
        assert!(!f.contains("11111111"));
    }

    #[test]
    fn test_role_filter() {
        let mut builder = RlsFilterBuilder::new();
        builder.add_rule("employees", patterns::role_filter(
            "department_id IN (SELECT id FROM user_departments WHERE user_id = '{{user_id}}')",
            vec!["manager", "hr_admin"]
        ));

        // User without matching role
        let ctx = create_context();
        let filter = builder.build_filter("employees", &ctx);
        assert!(filter.is_none());

        // User with matching role
        let mut ctx = create_context();
        ctx.roles = vec!["manager".to_string()];
        let filter = builder.build_filter("employees", &ctx);
        assert!(filter.is_some());
    }
}
