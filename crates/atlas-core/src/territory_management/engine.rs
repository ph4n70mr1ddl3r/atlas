//! Territory Management Engine
//!
//! Oracle Fusion CX Sales > Territory Management.
//!
//! Features:
//! - Territory definitions with dimensions (geographic, product, industry, customer)
//! - Territory hierarchy (parent/child territories)
//! - Territory member assignments (salespeople, teams)
//! - Territory rules for automatic lead/opportunity routing
//! - Territory quotas (revenue targets)
//! - Territory realignment (reassign accounts on territory changes)
//! - Territory dashboard with coverage and performance analytics
//!
//! Process:
//! 1. Define territory dimensions (geography, product line, industry, etc.)
//! 2. Create territories with dimension-based rules
//! 3. Build territory hierarchy (parent/child)
//! 4. Assign salespeople as territory members
//! 5. Set revenue quotas per territory
//! 6. Route leads/opportunities to territories via matching rules
//! 7. Monitor territory coverage and quota attainment

use atlas_shared::{
    AtlasError, AtlasResult,
};
use super::TerritoryManagementRepository;
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

/// Valid territory types
const VALID_TERRITORY_TYPES: &[&str] = &[
    "geography", "product", "industry", "customer", "hybrid",
];

/// Valid member roles
const VALID_MEMBER_ROLES: &[&str] = &[
    "owner", "member", "backup",
];

/// Valid routing rule entity types
const VALID_ROUTING_ENTITY_TYPES: &[&str] = &[
    "lead", "opportunity", "account", "contact",
];

/// Valid routing rule match operators
const VALID_MATCH_OPERATORS: &[&str] = &[
    "equals", "contains", "starts_with", "ends_with", "in", "not_null",
];

/// Territory Management Engine
pub struct TerritoryManagementEngine {
    repository: Arc<dyn TerritoryManagementRepository>,
}

impl TerritoryManagementEngine {
    pub fn new(repository: Arc<dyn TerritoryManagementRepository>) -> Self {
        Self { repository }
    }

    // ========================================================================
    // Territory CRUD
    // ========================================================================

    /// Create a new territory
    pub async fn create_territory(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        territory_type: &str,
        parent_id: Option<Uuid>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::Territory> {
        // Validate
        let code = code.trim().to_uppercase();
        if code.is_empty() || code.len() > 50 {
            return Err(AtlasError::ValidationFailed(
                "Territory code must be 1-50 characters".to_string(),
            ));
        }
        if name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Territory name is required".to_string()));
        }
        if !VALID_TERRITORY_TYPES.contains(&territory_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid territory type '{}'. Must be one of: {}",
                territory_type,
                VALID_TERRITORY_TYPES.join(", ")
            )));
        }

        // Validate effective dates
        if let (Some(from), Some(to)) = (effective_from, effective_to) {
            if from > to {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }

        // Validate parent exists
        if let Some(pid) = parent_id {
            self.repository.get_territory(pid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Parent territory {} not found", pid)))?;
        }

        // Check uniqueness
        if self.repository.get_territory_by_code(org_id, &code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Territory '{}' already exists", code)));
        }

        info!("Creating territory '{}' ({}) for org {}", name, code, org_id);
        self.repository.create_territory(
            org_id, &code, name, description, territory_type,
            parent_id, owner_id, owner_name, effective_from, effective_to, created_by,
        ).await
    }

    /// Get a territory by ID
    pub async fn get_territory(&self, id: Uuid) -> AtlasResult<Option<atlas_shared::Territory>> {
        self.repository.get_territory(id).await
    }

    /// Get a territory by code
    pub async fn get_territory_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<atlas_shared::Territory>> {
        self.repository.get_territory_by_code(org_id, code).await
    }

    /// List territories with optional filters
    pub async fn list_territories(
        &self,
        org_id: Uuid,
        territory_type: Option<&str>,
        parent_id: Option<Uuid>,
        include_inactive: bool,
    ) -> AtlasResult<Vec<atlas_shared::Territory>> {
        if let Some(tt) = territory_type {
            if !VALID_TERRITORY_TYPES.contains(&tt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid territory type '{}'", tt
                )));
            }
        }
        self.repository.list_territories(org_id, territory_type, parent_id, include_inactive).await
    }

    /// Update a territory
    pub async fn update_territory(
        &self,
        id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        territory_type: Option<&str>,
        parent_id: Option<Option<Uuid>>,
        owner_id: Option<Uuid>,
        owner_name: Option<&str>,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
    ) -> AtlasResult<atlas_shared::Territory> {
        let existing = self.repository.get_territory(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Territory {} not found", id)))?;

        if let Some(tt) = territory_type {
            if !VALID_TERRITORY_TYPES.contains(&tt) {
                return Err(AtlasError::ValidationFailed(format!(
                    "Invalid territory type '{}'", tt
                )));
            }
        }

        // Validate parent is not creating a cycle
        if let Some(Some(pid)) = parent_id {
            if pid == id {
                return Err(AtlasError::ValidationFailed(
                    "Territory cannot be its own parent".to_string(),
                ));
            }
            // Check parent exists
            self.repository.get_territory(pid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Parent territory {} not found", pid)))?;
        }

        // Validate dates
        let from = effective_from.or(existing.effective_from);
        let to = effective_to.or(existing.effective_to);
        if let (Some(f), Some(t)) = (from, to) {
            if f > t {
                return Err(AtlasError::ValidationFailed(
                    "effective_from must be before effective_to".to_string(),
                ));
            }
        }

        info!("Updating territory {} ({})", id, existing.code);
        self.repository.update_territory(
            id, name, description, territory_type, parent_id,
            owner_id, owner_name, effective_from, effective_to,
        ).await
    }

    /// Activate a territory
    pub async fn activate_territory(&self, id: Uuid) -> AtlasResult<atlas_shared::Territory> {
        let territory = self.repository.get_territory(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Territory {} not found", id)))?;
        if territory.is_active {
            return Err(AtlasError::ValidationFailed("Territory is already active".to_string()));
        }
        info!("Activating territory {}", territory.code);
        self.repository.update_territory_status(id, true).await
    }

    /// Deactivate a territory
    pub async fn deactivate_territory(&self, id: Uuid) -> AtlasResult<atlas_shared::Territory> {
        let territory = self.repository.get_territory(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Territory {} not found", id)))?;
        if !territory.is_active {
            return Err(AtlasError::ValidationFailed("Territory is already inactive".to_string()));
        }
        // Check for active children
        let children = self.repository.list_territories(
            territory.organization_id, None, Some(id), false,
        ).await?;
        if !children.is_empty() {
            return Err(AtlasError::ValidationFailed(format!(
                "Cannot deactivate territory with {} active child territories", children.len()
            )));
        }
        info!("Deactivating territory {}", territory.code);
        self.repository.update_territory_status(id, false).await
    }

    /// Delete a territory
    pub async fn delete_territory(&self, id: Uuid) -> AtlasResult<()> {
        let territory = self.repository.get_territory(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Territory {} not found", id)))?;

        // Check for children
        let children = self.repository.list_territories(
            territory.organization_id, None, Some(id), true,
        ).await?;
        if !children.is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Cannot delete territory with child territories. Remove children first.".to_string(),
            ));
        }

        info!("Deleting territory {} ({})", territory.code, id);
        self.repository.delete_territory(id).await
    }

    // ========================================================================
    // Territory Members
    // ========================================================================

    /// Add a member to a territory
    pub async fn add_territory_member(
        &self,
        org_id: Uuid,
        territory_id: Uuid,
        user_id: Uuid,
        user_name: &str,
        role: &str,
        effective_from: Option<chrono::NaiveDate>,
        effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TerritoryMember> {
        // Verify territory exists
        self.repository.get_territory(territory_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Territory {} not found", territory_id)))?;

        if user_name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Member name is required".to_string()));
        }

        if !VALID_MEMBER_ROLES.contains(&role) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid member role '{}'. Must be one of: {}",
                role,
                VALID_MEMBER_ROLES.join(", ")
            )));
        }

        // Check duplicate
        if self.repository.find_member(territory_id, user_id, role).await?.is_some() {
            return Err(AtlasError::Conflict(
                format!("User {} already has role '{}' in territory {}", user_id, role, territory_id)
            ));
        }

        info!("Adding member {} to territory {}", user_name, territory_id);
        self.repository.add_member(
            org_id, territory_id, user_id, user_name, role,
            effective_from, effective_to, created_by,
        ).await
    }

    /// List members of a territory
    pub async fn list_territory_members(
        &self,
        territory_id: Uuid,
        role: Option<&str>,
    ) -> AtlasResult<Vec<atlas_shared::TerritoryMember>> {
        if let Some(r) = role {
            if !VALID_MEMBER_ROLES.contains(&r) {
                return Err(AtlasError::ValidationFailed(format!("Invalid role '{}'", r)));
            }
        }
        self.repository.list_members(territory_id, role).await
    }

    /// Remove a member from a territory
    pub async fn remove_territory_member(&self, member_id: Uuid) -> AtlasResult<()> {
        self.repository.remove_member(member_id).await
    }

    // ========================================================================
    // Territory Rules (Routing)
    // ========================================================================

    /// Add a routing rule to a territory
    pub async fn add_territory_rule(
        &self,
        org_id: Uuid,
        territory_id: Uuid,
        entity_type: &str,
        field_name: &str,
        match_operator: &str,
        match_value: &str,
        priority: i32,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TerritoryRule> {
        // Verify territory exists
        self.repository.get_territory(territory_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Territory {} not found", territory_id)))?;

        if !VALID_ROUTING_ENTITY_TYPES.contains(&entity_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid entity type '{}'. Must be one of: {}",
                entity_type,
                VALID_ROUTING_ENTITY_TYPES.join(", ")
            )));
        }

        if field_name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Field name is required".to_string()));
        }

        if !VALID_MATCH_OPERATORS.contains(&match_operator) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid match operator '{}'. Must be one of: {}",
                match_operator,
                VALID_MATCH_OPERATORS.join(", ")
            )));
        }

        if match_operator != "not_null" && match_value.trim().is_empty() {
            return Err(AtlasError::ValidationFailed(
                "Match value is required for this operator".to_string(),
            ));
        }

        if priority < 1 {
            return Err(AtlasError::ValidationFailed("Priority must be >= 1".to_string()));
        }

        info!("Adding routing rule to territory {}: {} {} {}", territory_id, field_name, match_operator, match_value);
        self.repository.add_rule(
            org_id, territory_id, entity_type, field_name,
            match_operator, match_value, priority, created_by,
        ).await
    }

    /// List routing rules for a territory
    pub async fn list_territory_rules(
        &self,
        territory_id: Uuid,
        entity_type: Option<&str>,
    ) -> AtlasResult<Vec<atlas_shared::TerritoryRule>> {
        self.repository.list_rules(territory_id, entity_type).await
    }

    /// Remove a routing rule
    pub async fn remove_territory_rule(&self, rule_id: Uuid) -> AtlasResult<()> {
        self.repository.remove_rule(rule_id).await
    }

    /// Route an entity (lead/opportunity/account) to the best matching territory
    pub async fn route_entity(
        &self,
        org_id: Uuid,
        entity_type: &str,
        entity_data: &serde_json::Value,
    ) -> AtlasResult<RouteResult> {
        if !VALID_ROUTING_ENTITY_TYPES.contains(&entity_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid entity type '{}'. Must be one of: {}",
                entity_type,
                VALID_ROUTING_ENTITY_TYPES.join(", ")
            )));
        }

        let territories = self.repository.list_territories(org_id, None, None, false).await?;
        let mut matches: Vec<RouteMatch> = Vec::new();

        for territory in &territories {
            let rules = self.repository.list_rules(territory.id, Some(entity_type)).await?;
            if rules.is_empty() {
                continue;
            }

            let mut all_match = true;
            let mut matched_rules = Vec::new();

            for rule in &rules {
                let field_val = entity_data.get(&rule.field_name)
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let matched = match rule.match_operator.as_str() {
                    "equals" => field_val.eq_ignore_ascii_case(&rule.match_value),
                    "contains" => field_val.to_lowercase().contains(&rule.match_value.to_lowercase()),
                    "starts_with" => field_val.to_lowercase().starts_with(&rule.match_value.to_lowercase()),
                    "ends_with" => field_val.to_lowercase().ends_with(&rule.match_value.to_lowercase()),
                    "in" => {
                        let values: Vec<&str> = rule.match_value.split(',').map(|s| s.trim()).collect();
                        values.iter().any(|v| v.eq_ignore_ascii_case(field_val))
                    },
                    "not_null" => !field_val.is_empty(),
                    _ => false,
                };

                if matched {
                    matched_rules.push(rule.id);
                } else {
                    all_match = false;
                    break;
                }
            }

            if all_match && !matched_rules.is_empty() {
                let best_priority = rules.iter().map(|r| r.priority).min().unwrap_or(999);
                matches.push(RouteMatch {
                    territory_id: territory.id,
                    territory_code: territory.code.clone(),
                    territory_name: territory.name.clone(),
                    matched_rules,
                    priority: best_priority,
                });
            }
        }

        // Sort by priority (lower = better match)
        matches.sort_by_key(|m| m.priority);

        let best = matches.first().map(|m| m.clone());
        Ok(RouteResult {
            matched: best.is_some(),
            best_match: best,
            all_matches: matches,
        })
    }

    // ========================================================================
    // Territory Quotas
    // ========================================================================

    /// Set a revenue quota for a territory
    pub async fn set_territory_quota(
        &self,
        org_id: Uuid,
        territory_id: Uuid,
        period_name: &str,
        period_start: chrono::NaiveDate,
        period_end: chrono::NaiveDate,
        revenue_quota: &str,
        currency_code: &str,
        created_by: Option<Uuid>,
    ) -> AtlasResult<atlas_shared::TerritoryQuota> {
        // Verify territory exists
        self.repository.get_territory(territory_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Territory {} not found", territory_id)))?;

        if period_name.trim().is_empty() {
            return Err(AtlasError::ValidationFailed("Period name is required".to_string()));
        }
        if period_start >= period_end {
            return Err(AtlasError::ValidationFailed("Period start must be before period end".to_string()));
        }

        let quota: f64 = revenue_quota.parse().map_err(|_| {
            AtlasError::ValidationFailed("Revenue quota must be a valid number".to_string())
        })?;
        if quota < 0.0 {
            return Err(AtlasError::ValidationFailed("Revenue quota cannot be negative".to_string()));
        }

        // Check for existing quota for same period
        if let Some(existing) = self.repository.find_quota(territory_id, period_name).await? {
            info!("Updating existing quota {} for territory {}", existing.id, territory_id);
            return self.repository.update_quota_amount(existing.id, revenue_quota).await;
        }

        info!("Setting quota for territory {} period {}: {}", territory_id, period_name, revenue_quota);
        self.repository.create_quota(
            org_id, territory_id, period_name, period_start, period_end,
            revenue_quota, "0", currency_code, created_by,
        ).await
    }

    /// Get a quota by ID
    pub async fn get_territory_quota(&self, id: Uuid) -> AtlasResult<Option<atlas_shared::TerritoryQuota>> {
        self.repository.get_quota(id).await
    }

    /// List quotas for a territory
    pub async fn list_territory_quotas(
        &self,
        territory_id: Uuid,
    ) -> AtlasResult<Vec<atlas_shared::TerritoryQuota>> {
        self.repository.list_quotas(territory_id).await
    }

    /// Update quota attainment (actual revenue)
    pub async fn update_quota_attainment(
        &self,
        quota_id: Uuid,
        actual_revenue: &str,
    ) -> AtlasResult<atlas_shared::TerritoryQuota> {
        let actual: f64 = actual_revenue.parse().map_err(|_| {
            AtlasError::ValidationFailed("Actual revenue must be a valid number".to_string())
        })?;
        if actual < 0.0 {
            return Err(AtlasError::ValidationFailed("Actual revenue cannot be negative".to_string()));
        }
        self.repository.update_quota_actual(quota_id, actual_revenue).await
    }

    /// Delete a quota
    pub async fn delete_territory_quota(&self, quota_id: Uuid) -> AtlasResult<()> {
        self.repository.delete_quota(quota_id).await
    }

    // ========================================================================
    // Dashboard
    // ========================================================================

    /// Get the territory management dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<atlas_shared::TerritoryDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

/// Result of routing an entity to territories
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResult {
    pub matched: bool,
    pub best_match: Option<RouteMatch>,
    pub all_matches: Vec<RouteMatch>,
}

/// A single territory match from routing
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteMatch {
    pub territory_id: Uuid,
    pub territory_code: String,
    pub territory_name: String,
    pub matched_rules: Vec<Uuid>,
    pub priority: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_territory_types() {
        assert!(VALID_TERRITORY_TYPES.contains(&"geography"));
        assert!(VALID_TERRITORY_TYPES.contains(&"product"));
        assert!(VALID_TERRITORY_TYPES.contains(&"industry"));
        assert!(VALID_TERRITORY_TYPES.contains(&"customer"));
        assert!(VALID_TERRITORY_TYPES.contains(&"hybrid"));
        assert!(!VALID_TERRITORY_TYPES.contains(&"unknown"));
    }

    #[test]
    fn test_valid_member_roles() {
        assert!(VALID_MEMBER_ROLES.contains(&"owner"));
        assert!(VALID_MEMBER_ROLES.contains(&"member"));
        assert!(VALID_MEMBER_ROLES.contains(&"backup"));
        assert!(!VALID_MEMBER_ROLES.contains(&"admin"));
    }

    #[test]
    fn test_valid_routing_entity_types() {
        assert!(VALID_ROUTING_ENTITY_TYPES.contains(&"lead"));
        assert!(VALID_ROUTING_ENTITY_TYPES.contains(&"opportunity"));
        assert!(VALID_ROUTING_ENTITY_TYPES.contains(&"account"));
        assert!(VALID_ROUTING_ENTITY_TYPES.contains(&"contact"));
        assert!(!VALID_ROUTING_ENTITY_TYPES.contains(&"invoice"));
    }

    #[test]
    fn test_valid_match_operators() {
        assert!(VALID_MATCH_OPERATORS.contains(&"equals"));
        assert!(VALID_MATCH_OPERATORS.contains(&"contains"));
        assert!(VALID_MATCH_OPERATORS.contains(&"starts_with"));
        assert!(VALID_MATCH_OPERATORS.contains(&"ends_with"));
        assert!(VALID_MATCH_OPERATORS.contains(&"in"));
        assert!(VALID_MATCH_OPERATORS.contains(&"not_null"));
        assert!(!VALID_MATCH_OPERATORS.contains(&"regex"));
    }

    #[test]
    fn test_match_operator_equals() {
        let field_val = "California";
        let match_value = "california";
        assert!(field_val.eq_ignore_ascii_case(match_value));
    }

    #[test]
    fn test_match_operator_contains() {
        let field_val = "San Francisco";
        let match_value = "francisco";
        assert!(field_val.to_lowercase().contains(&match_value.to_lowercase()));
    }

    #[test]
    fn test_match_operator_in() {
        let field_val = "NY";
        let match_value = "CA,NY,TX";
        let values: Vec<&str> = match_value.split(',').map(|s| s.trim()).collect();
        assert!(values.iter().any(|v| v.eq_ignore_ascii_case(field_val)));
    }

    #[test]
    fn test_priority_sorting() {
        let mut matches = vec![
            RouteMatch {
                territory_id: Uuid::new_v4(),
                territory_code: "T3".to_string(),
                territory_name: "Third".to_string(),
                matched_rules: vec![],
                priority: 30,
            },
            RouteMatch {
                territory_id: Uuid::new_v4(),
                territory_code: "T1".to_string(),
                territory_name: "First".to_string(),
                matched_rules: vec![],
                priority: 10,
            },
            RouteMatch {
                territory_id: Uuid::new_v4(),
                territory_code: "T2".to_string(),
                territory_name: "Second".to_string(),
                matched_rules: vec![],
                priority: 20,
            },
        ];
        matches.sort_by_key(|m| m.priority);
        assert_eq!(matches[0].priority, 10);
        assert_eq!(matches[1].priority, 20);
        assert_eq!(matches[2].priority, 30);
    }

    #[test]
    fn test_quota_attainment_percent() {
        let quota = 100000.0_f64;
        let actual = 75000.0_f64;
        let pct = (actual / quota) * 100.0;
        assert!((pct - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_code_validation() {
        let code = "".to_string();
        assert!(code.trim().is_empty());
        let code = "A".repeat(51);
        assert!(code.len() > 50);
        let code = "WEST-COAST".to_string();
        let upper = code.to_uppercase();
        assert_eq!(upper, "WEST-COAST");
        assert!(!upper.is_empty() && upper.len() <= 50);
    }
}
