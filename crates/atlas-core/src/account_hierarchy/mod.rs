//! Account Hierarchy Module
//!
//! Oracle Fusion Cloud ERP-inspired Chart of Account Hierarchies.
//! Manages hierarchical rollup structures for financial reporting,
//! budgeting, and analysis. Supports multiple hierarchy versions.
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Chart of Accounts > Account Hierarchies

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::info;

// ============================================================================
// Constants
// ============================================================================

const VALID_HIERARCHY_TYPES: &[&str] = &["account", "cost_center", "entity", "product", "project", "intercompany", "custom"];
const VALID_NODE_TYPES: &[&str] = &["root", "summary", "detail"];
#[allow(dead_code)]
const VALID_STATUSES: &[&str] = &["draft", "active", "inactive"];

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountHierarchy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub hierarchy_type: String,
    pub version: i32,
    pub status: String,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyNode {
    pub id: Uuid,
    pub hierarchy_id: Uuid,
    pub organization_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub account_code: String,
    pub account_name: Option<String>,
    pub node_type: String,
    pub display_order: i32,
    pub level_depth: i32,
    pub is_enabled: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyDashboard {
    pub organization_id: Uuid,
    pub total_hierarchies: i32,
    pub active_hierarchies: i32,
    pub total_nodes: i32,
    pub by_type: serde_json::Value,
}

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait AccountHierarchyRepository: Send + Sync {
    async fn create_hierarchy(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, hierarchy_type: &str, effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>, created_by: Option<Uuid>) -> AtlasResult<AccountHierarchy>;
    async fn get_hierarchy(&self, id: Uuid) -> AtlasResult<Option<AccountHierarchy>>;
    async fn get_hierarchy_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountHierarchy>>;
    async fn list_hierarchies(&self, org_id: Uuid, hierarchy_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<AccountHierarchy>>;
    async fn update_hierarchy_status(&self, id: Uuid, status: &str, is_active: bool) -> AtlasResult<AccountHierarchy>;
    async fn increment_version(&self, id: Uuid) -> AtlasResult<AccountHierarchy>;

    async fn add_node(&self, hierarchy_id: Uuid, org_id: Uuid, parent_id: Option<Uuid>, account_code: &str, account_name: Option<&str>, node_type: &str, display_order: i32, level_depth: i32) -> AtlasResult<HierarchyNode>;
    async fn get_node(&self, id: Uuid) -> AtlasResult<Option<HierarchyNode>>;
    async fn list_nodes(&self, hierarchy_id: Uuid) -> AtlasResult<Vec<HierarchyNode>>;
    async fn list_children(&self, parent_id: Uuid) -> AtlasResult<Vec<HierarchyNode>>;
    async fn update_node(&self, id: Uuid, account_name: Option<&str>, display_order: Option<i32>, is_enabled: Option<bool>) -> AtlasResult<HierarchyNode>;
    async fn move_node(&self, id: Uuid, new_parent_id: Option<Uuid>, new_display_order: i32, new_depth: i32) -> AtlasResult<HierarchyNode>;
    async fn remove_node(&self, id: Uuid) -> AtlasResult<()>;

    async fn get_ancestors(&self, node_id: Uuid) -> AtlasResult<Vec<HierarchyNode>>;
    async fn get_descendants(&self, node_id: Uuid) -> AtlasResult<Vec<HierarchyNode>>;

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<HierarchyDashboard>;
}

/// PostgreSQL stub
#[allow(dead_code)]
pub struct PostgresAccountHierarchyRepository { #[allow(dead_code)]
    pool: PgPool }
impl PostgresAccountHierarchyRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl AccountHierarchyRepository for PostgresAccountHierarchyRepository {
    async fn create_hierarchy(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: &str, _: Option<chrono::NaiveDate>, _: Option<chrono::NaiveDate>, _: Option<Uuid>) -> AtlasResult<AccountHierarchy> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_hierarchy(&self, _: Uuid) -> AtlasResult<Option<AccountHierarchy>> { Ok(None) }
    async fn get_hierarchy_by_code(&self, _: Uuid, _: &str) -> AtlasResult<Option<AccountHierarchy>> { Ok(None) }
    async fn list_hierarchies(&self, _: Uuid, _: Option<&str>, _: Option<bool>) -> AtlasResult<Vec<AccountHierarchy>> { Ok(vec![]) }
    async fn update_hierarchy_status(&self, _: Uuid, _: &str, _: bool) -> AtlasResult<AccountHierarchy> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn increment_version(&self, _: Uuid) -> AtlasResult<AccountHierarchy> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn add_node(&self, _: Uuid, _: Uuid, _: Option<Uuid>, _: &str, _: Option<&str>, _: &str, _: i32, _: i32) -> AtlasResult<HierarchyNode> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get_node(&self, _: Uuid) -> AtlasResult<Option<HierarchyNode>> { Ok(None) }
    async fn list_nodes(&self, _: Uuid) -> AtlasResult<Vec<HierarchyNode>> { Ok(vec![]) }
    async fn list_children(&self, _: Uuid) -> AtlasResult<Vec<HierarchyNode>> { Ok(vec![]) }
    async fn update_node(&self, _: Uuid, _: Option<&str>, _: Option<i32>, _: Option<bool>) -> AtlasResult<HierarchyNode> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn move_node(&self, _: Uuid, _: Option<Uuid>, _: i32, _: i32) -> AtlasResult<HierarchyNode> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn remove_node(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn get_ancestors(&self, _: Uuid) -> AtlasResult<Vec<HierarchyNode>> { Ok(vec![]) }
    async fn get_descendants(&self, _: Uuid) -> AtlasResult<Vec<HierarchyNode>> { Ok(vec![]) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<HierarchyDashboard> {
        Ok(HierarchyDashboard { organization_id: org_id, total_hierarchies: 0, active_hierarchies: 0, total_nodes: 0, by_type: serde_json::json!([]) })
    }
}

// ============================================================================
// Engine
// ============================================================================

pub struct AccountHierarchyEngine {
    repository: Arc<dyn AccountHierarchyRepository>,
}

impl AccountHierarchyEngine {
    pub fn new(repository: Arc<dyn AccountHierarchyRepository>) -> Self { Self { repository } }

    // ── Hierarchy operations ──

    pub async fn create_hierarchy(
        &self, org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        hierarchy_type: &str, effective_from: Option<chrono::NaiveDate>, effective_to: Option<chrono::NaiveDate>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<AccountHierarchy> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed("Code and name are required".into()));
        }
        if !VALID_HIERARCHY_TYPES.contains(&hierarchy_type) {
            return Err(AtlasError::ValidationFailed(format!(
                "Invalid hierarchy type '{}'. Must be one of: {}", hierarchy_type, VALID_HIERARCHY_TYPES.join(", ")
            )));
        }
        if let Some(to) = effective_to {
            if let Some(from) = effective_from {
                if to < from {
                    return Err(AtlasError::ValidationFailed("Effective to must be after effective from".into()));
                }
            }
        }
        if self.repository.get_hierarchy_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Hierarchy '{}' already exists", code)));
        }
        info!("Creating account hierarchy '{}' for org {}", code, org_id);
        self.repository.create_hierarchy(org_id, code, name, description, hierarchy_type, effective_from, effective_to, created_by).await
    }

    pub async fn get_hierarchy(&self, id: Uuid) -> AtlasResult<Option<AccountHierarchy>> { self.repository.get_hierarchy(id).await }
    pub async fn get_hierarchy_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountHierarchy>> { self.repository.get_hierarchy_by_code(org_id, code).await }

    pub async fn list_hierarchies(&self, org_id: Uuid, hierarchy_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<AccountHierarchy>> {
        if let Some(ht) = hierarchy_type {
            if !VALID_HIERARCHY_TYPES.contains(&ht) {
                return Err(AtlasError::ValidationFailed(format!("Invalid hierarchy type '{}'", ht)));
            }
        }
        self.repository.list_hierarchies(org_id, hierarchy_type, is_active).await
    }

    pub async fn activate(&self, id: Uuid) -> AtlasResult<AccountHierarchy> {
        let h = self.repository.get_hierarchy(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Hierarchy {} not found", id)))?;
        if h.is_active { return Err(AtlasError::ValidationFailed("Already active".into())); }
        info!("Activating hierarchy {}", h.code);
        self.repository.update_hierarchy_status(id, "active", true).await
    }

    pub async fn deactivate(&self, id: Uuid) -> AtlasResult<AccountHierarchy> {
        let h = self.repository.get_hierarchy(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Hierarchy {} not found", id)))?;
        if !h.is_active { return Err(AtlasError::ValidationFailed("Already inactive".into())); }
        info!("Deactivating hierarchy {}", h.code);
        self.repository.update_hierarchy_status(id, "inactive", false).await
    }

    // ── Node operations ──

    pub async fn add_node(
        &self, hierarchy_id: Uuid, org_id: Uuid, parent_id: Option<Uuid>,
        account_code: &str, account_name: Option<&str>, node_type: &str, display_order: i32,
    ) -> AtlasResult<HierarchyNode> {
        let h = self.repository.get_hierarchy(hierarchy_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Hierarchy {} not found", hierarchy_id)))?;
        if !h.is_active {
            return Err(AtlasError::ValidationFailed("Cannot add nodes to inactive hierarchy".into()));
        }
        if account_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Account code is required".into()));
        }
        if !VALID_NODE_TYPES.contains(&node_type) {
            return Err(AtlasError::ValidationFailed(format!("Invalid node type '{}'", node_type)));
        }

        let level_depth = if let Some(pid) = parent_id {
            let parent = self.repository.get_node(pid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Parent node {} not found", pid)))?;
            if parent.hierarchy_id != hierarchy_id {
                return Err(AtlasError::ValidationFailed("Parent must belong to same hierarchy".into()));
            }
            parent.level_depth + 1
        } else {
            0
        };

        // Validate root: only one root per hierarchy
        if parent_id.is_none() {
            let existing = self.repository.list_nodes(hierarchy_id).await?;
            if existing.iter().any(|n| n.parent_id.is_none()) {
                return Err(AtlasError::ValidationFailed("Hierarchy already has a root node".into()));
            }
        }

        info!("Adding node {} to hierarchy {}", account_code, h.code);
        self.repository.add_node(hierarchy_id, org_id, parent_id, account_code, account_name, node_type, display_order, level_depth).await
    }

    pub async fn get_node(&self, id: Uuid) -> AtlasResult<Option<HierarchyNode>> { self.repository.get_node(id).await }
    pub async fn list_nodes(&self, hierarchy_id: Uuid) -> AtlasResult<Vec<HierarchyNode>> { self.repository.list_nodes(hierarchy_id).await }
    pub async fn list_children(&self, parent_id: Uuid) -> AtlasResult<Vec<HierarchyNode>> { self.repository.list_children(parent_id).await }

    pub async fn update_node(&self, id: Uuid, account_name: Option<&str>, display_order: Option<i32>, is_enabled: Option<bool>) -> AtlasResult<HierarchyNode> {
        self.repository.get_node(id).await?.ok_or_else(|| AtlasError::EntityNotFound(format!("Node {} not found", id)))?;
        self.repository.update_node(id, account_name, display_order, is_enabled).await
    }

    pub async fn move_node(&self, id: Uuid, new_parent_id: Option<Uuid>, new_display_order: i32) -> AtlasResult<HierarchyNode> {
        let node = self.repository.get_node(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Node {} not found", id)))?;
        if let Some(pid) = new_parent_id {
            let parent = self.repository.get_node(pid).await?
                .ok_or_else(|| AtlasError::EntityNotFound(format!("Parent {} not found", pid)))?;
            if parent.hierarchy_id != node.hierarchy_id {
                return Err(AtlasError::ValidationFailed("Cannot move across hierarchies".into()));
            }
            // Prevent circular: new parent cannot be a descendant of the node
            let descendants = self.repository.get_descendants(id).await?;
            if descendants.iter().any(|d| d.id == pid) {
                return Err(AtlasError::ValidationFailed("Cannot move node under its own descendant".into()));
            }
        }
        let new_depth = if let Some(pid) = new_parent_id {
            let parent = self.repository.get_node(pid).await?.unwrap();
            parent.level_depth + 1
        } else { 0 };
        info!("Moving node {} to new parent", node.account_code);
        self.repository.move_node(id, new_parent_id, new_display_order, new_depth).await
    }

    pub async fn remove_node(&self, id: Uuid) -> AtlasResult<()> {
        let node = self.repository.get_node(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Node {} not found", id)))?;
        let children = self.repository.list_children(id).await?;
        if !children.is_empty() {
            return Err(AtlasError::ValidationFailed("Cannot remove node with children. Remove children first".into()));
        }
        info!("Removing node {} from hierarchy", node.account_code);
        self.repository.remove_node(id).await
    }

    pub async fn get_ancestors(&self, node_id: Uuid) -> AtlasResult<Vec<HierarchyNode>> {
        self.repository.get_ancestors(node_id).await
    }

    pub async fn get_descendants(&self, node_id: Uuid) -> AtlasResult<Vec<HierarchyNode>> {
        self.repository.get_descendants(node_id).await
    }

    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<HierarchyDashboard> {
        self.repository.get_dashboard(org_id).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo {
        hierarchies: std::sync::Mutex<Vec<AccountHierarchy>>,
        nodes: std::sync::Mutex<Vec<HierarchyNode>>,
    }
    impl MockRepo { fn new() -> Self { Self { hierarchies: std::sync::Mutex::new(vec![]), nodes: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl AccountHierarchyRepository for MockRepo {
        async fn create_hierarchy(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, hierarchy_type: &str, eff_from: Option<chrono::NaiveDate>, eff_to: Option<chrono::NaiveDate>, created_by: Option<Uuid>) -> AtlasResult<AccountHierarchy> {
            let h = AccountHierarchy {
                id: Uuid::new_v4(), organization_id: org_id, code: code.into(), name: name.into(),
                description: description.map(Into::into), hierarchy_type: hierarchy_type.into(),
                version: 1, status: "draft".into(), is_active: true, effective_from: eff_from, effective_to: eff_to,
                metadata: serde_json::json!({}), created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.hierarchies.lock().unwrap().push(h.clone());
            Ok(h)
        }
        async fn get_hierarchy(&self, id: Uuid) -> AtlasResult<Option<AccountHierarchy>> {
            Ok(self.hierarchies.lock().unwrap().iter().find(|h| h.id == id).cloned())
        }
        async fn get_hierarchy_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<AccountHierarchy>> {
            Ok(self.hierarchies.lock().unwrap().iter().find(|h| h.organization_id == org_id && h.code == code).cloned())
        }
        async fn list_hierarchies(&self, org_id: Uuid, hierarchy_type: Option<&str>, is_active: Option<bool>) -> AtlasResult<Vec<AccountHierarchy>> {
            Ok(self.hierarchies.lock().unwrap().iter()
                .filter(|h| h.organization_id == org_id && (hierarchy_type.is_none() || h.hierarchy_type == hierarchy_type.unwrap()) && (is_active.is_none() || h.is_active == is_active.unwrap()))
                .cloned().collect())
        }
        async fn update_hierarchy_status(&self, id: Uuid, status: &str, is_active: bool) -> AtlasResult<AccountHierarchy> {
            let mut all = self.hierarchies.lock().unwrap();
            let h = all.iter_mut().find(|h| h.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            h.status = status.into(); h.is_active = is_active; h.updated_at = Utc::now(); Ok(h.clone())
        }
        async fn increment_version(&self, id: Uuid) -> AtlasResult<AccountHierarchy> {
            let mut all = self.hierarchies.lock().unwrap();
            let h = all.iter_mut().find(|h| h.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            h.version += 1; h.updated_at = Utc::now(); Ok(h.clone())
        }
        async fn add_node(&self, hierarchy_id: Uuid, org_id: Uuid, parent_id: Option<Uuid>, account_code: &str, account_name: Option<&str>, node_type: &str, display_order: i32, level_depth: i32) -> AtlasResult<HierarchyNode> {
            let n = HierarchyNode {
                id: Uuid::new_v4(), hierarchy_id, organization_id: org_id, parent_id,
                account_code: account_code.into(), account_name: account_name.map(Into::into),
                node_type: node_type.into(), display_order, level_depth, is_enabled: true,
                metadata: serde_json::json!({}), created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.nodes.lock().unwrap().push(n.clone());
            Ok(n)
        }
        async fn get_node(&self, id: Uuid) -> AtlasResult<Option<HierarchyNode>> {
            Ok(self.nodes.lock().unwrap().iter().find(|n| n.id == id).cloned())
        }
        async fn list_nodes(&self, hierarchy_id: Uuid) -> AtlasResult<Vec<HierarchyNode>> {
            Ok(self.nodes.lock().unwrap().iter().filter(|n| n.hierarchy_id == hierarchy_id).cloned().collect())
        }
        async fn list_children(&self, parent_id: Uuid) -> AtlasResult<Vec<HierarchyNode>> {
            Ok(self.nodes.lock().unwrap().iter().filter(|n| n.parent_id == Some(parent_id)).cloned().collect())
        }
        async fn update_node(&self, id: Uuid, account_name: Option<&str>, display_order: Option<i32>, is_enabled: Option<bool>) -> AtlasResult<HierarchyNode> {
            let mut all = self.nodes.lock().unwrap();
            let n = all.iter_mut().find(|n| n.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            if let Some(name) = account_name { n.account_name = Some(name.into()); }
            if let Some(order) = display_order { n.display_order = order; }
            if let Some(enabled) = is_enabled { n.is_enabled = enabled; }
            n.updated_at = Utc::now(); Ok(n.clone())
        }
        async fn move_node(&self, id: Uuid, new_parent_id: Option<Uuid>, new_display_order: i32, new_depth: i32) -> AtlasResult<HierarchyNode> {
            let mut all = self.nodes.lock().unwrap();
            let n = all.iter_mut().find(|n| n.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            n.parent_id = new_parent_id; n.display_order = new_display_order; n.level_depth = new_depth;
            n.updated_at = Utc::now(); Ok(n.clone())
        }
        async fn remove_node(&self, id: Uuid) -> AtlasResult<()> {
            self.nodes.lock().unwrap().retain(|n| n.id != id); Ok(())
        }
        async fn get_ancestors(&self, node_id: Uuid) -> AtlasResult<Vec<HierarchyNode>> {
            let all = self.nodes.lock().unwrap();
            let mut ancestors = vec![];
            let mut current = all.iter().find(|n| n.id == node_id).cloned();
            while let Some(node) = current {
                if let Some(pid) = node.parent_id {
                    let parent = all.iter().find(|n| n.id == pid).cloned();
                    if let Some(p) = &parent { ancestors.push(p.clone()); }
                    current = parent;
                } else { break; }
            }
            Ok(ancestors)
        }
        async fn get_descendants(&self, node_id: Uuid) -> AtlasResult<Vec<HierarchyNode>> {
            let all = self.nodes.lock().unwrap();
            let mut desc = vec![];
            let mut queue = vec![node_id];
            while let Some(cid) = queue.pop() {
                let children: Vec<_> = all.iter().filter(|n| n.parent_id == Some(cid)).cloned().collect();
                for c in &children { queue.push(c.id); }
                desc.extend(children);
            }
            Ok(desc)
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<HierarchyDashboard> {
            let hierarchies = self.hierarchies.lock().unwrap();
            let nodes = self.nodes.lock().unwrap();
            let org_h: Vec<_> = hierarchies.iter().filter(|h| h.organization_id == org_id).collect();
            let org_n = nodes.iter().filter(|n| org_h.iter().any(|h| h.id == n.hierarchy_id)).count() as i32;
            Ok(HierarchyDashboard {
                organization_id: org_id, total_hierarchies: org_h.len() as i32,
                active_hierarchies: org_h.iter().filter(|h| h.is_active).count() as i32,
                total_nodes: org_n, by_type: serde_json::json!([]),
            })
        }
    }

    fn eng() -> AccountHierarchyEngine { AccountHierarchyEngine::new(Arc::new(MockRepo::new())) }
    fn today() -> chrono::NaiveDate { chrono::Utc::now().date_naive() }

    #[test]
    fn test_valid_hierarchy_types() {
        assert_eq!(VALID_HIERARCHY_TYPES.len(), 7);
        assert!(VALID_HIERARCHY_TYPES.contains(&"account"));
        assert!(VALID_HIERARCHY_TYPES.contains(&"cost_center"));
    }

    #[test]
    fn test_valid_node_types() {
        assert_eq!(VALID_NODE_TYPES.len(), 3);
    }

    #[tokio::test]
    async fn test_create_hierarchy_valid() {
        let h = eng().create_hierarchy(Uuid::new_v4(), "COA", "Chart of Accounts", Some("Full COA"), "account", Some(today()), None, None).await.unwrap();
        assert_eq!(h.code, "COA");
        assert!(h.is_active);
        assert_eq!(h.version, 1);
    }

    #[tokio::test]
    async fn test_create_hierarchy_empty_code() {
        assert!(eng().create_hierarchy(Uuid::new_v4(), "", "Name", None, "account", None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_hierarchy_invalid_type() {
        assert!(eng().create_hierarchy(Uuid::new_v4(), "CODE", "Name", None, "invalid", None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_hierarchy_duplicate() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        assert!(e.create_hierarchy(org, "COA", "Chart 2", None, "account", None, None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_hierarchy_invalid_dates() {
        assert!(eng().create_hierarchy(Uuid::new_v4(), "COA", "Chart", None, "account", Some(today()), Some(chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()), None).await.is_err());
    }

    #[tokio::test]
    async fn test_activate_deactivate() {
        let e = eng();
        let h = e.create_hierarchy(Uuid::new_v4(), "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let _ = e.deactivate(h.id).await.unwrap();
        let active = e.activate(h.id).await.unwrap();
        assert!(active.is_active);
    }

    #[tokio::test]
    async fn test_deactivate_already_inactive() {
        let e = eng();
        let h = e.create_hierarchy(Uuid::new_v4(), "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let _ = e.deactivate(h.id).await.unwrap();
        assert!(e.deactivate(h.id).await.is_err());
    }

    #[tokio::test]
    async fn test_activate_already_active() {
        let e = eng();
        let h = e.create_hierarchy(Uuid::new_v4(), "COA", "Chart", None, "account", None, None, None).await.unwrap();
        assert!(e.activate(h.id).await.is_err());
    }

    #[tokio::test]
    async fn test_add_root_node() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let root = e.add_node(h.id, org, None, "0000", Some("All Accounts"), "root", 1).await.unwrap();
        assert_eq!(root.account_code, "0000");
        assert_eq!(root.level_depth, 0);
        assert!(root.parent_id.is_none());
    }

    #[tokio::test]
    async fn test_add_child_node() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let root = e.add_node(h.id, org, None, "0000", Some("All"), "root", 1).await.unwrap();
        let child = e.add_node(h.id, org, Some(root.id), "1000", Some("Assets"), "summary", 1).await.unwrap();
        assert_eq!(child.level_depth, 1);
        assert_eq!(child.parent_id.unwrap(), root.id);
    }

    #[tokio::test]
    async fn test_add_node_empty_code() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        assert!(e.add_node(h.id, org, None, "", None, "root", 1).await.is_err());
    }

    #[tokio::test]
    async fn test_add_node_invalid_type() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        assert!(e.add_node(h.id, org, None, "0000", None, "invalid", 1).await.is_err());
    }

    #[tokio::test]
    async fn test_add_node_inactive_hierarchy() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let _ = e.deactivate(h.id).await.unwrap();
        assert!(e.add_node(h.id, org, None, "0000", None, "root", 1).await.is_err());
    }

    #[tokio::test]
    async fn test_add_node_duplicate_root() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let _ = e.add_node(h.id, org, None, "0000", None, "root", 1).await.unwrap();
        assert!(e.add_node(h.id, org, None, "0001", None, "root", 2).await.is_err());
    }

    #[tokio::test]
    async fn test_remove_node_with_children() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let root = e.add_node(h.id, org, None, "0000", None, "root", 1).await.unwrap();
        let _ = e.add_node(h.id, org, Some(root.id), "1000", None, "summary", 1).await.unwrap();
        assert!(e.remove_node(root.id).await.is_err());
    }

    #[tokio::test]
    async fn test_remove_leaf_node() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let root = e.add_node(h.id, org, None, "0000", None, "root", 1).await.unwrap();
        let leaf = e.add_node(h.id, org, Some(root.id), "1000", Some("Assets"), "summary", 1).await.unwrap();
        e.remove_node(leaf.id).await.unwrap();
        let children = e.list_children(root.id).await.unwrap();
        assert!(children.is_empty());
    }

    #[tokio::test]
    async fn test_update_node() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let root = e.add_node(h.id, org, None, "0000", Some("Old"), "root", 1).await.unwrap();
        let updated = e.update_node(root.id, Some("All Accounts"), Some(2), None).await.unwrap();
        assert_eq!(updated.account_name.unwrap(), "All Accounts");
        assert_eq!(updated.display_order, 2);
    }

    #[tokio::test]
    async fn test_move_node() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let root = e.add_node(h.id, org, None, "0000", Some("All"), "root", 1).await.unwrap();
        let child1 = e.add_node(h.id, org, Some(root.id), "1000", Some("Assets"), "summary", 1).await.unwrap();
        let child2 = e.add_node(h.id, org, Some(root.id), "2000", Some("Liabilities"), "summary", 2).await.unwrap();
        let moved = e.move_node(child1.id, Some(child2.id), 1).await.unwrap();
        assert_eq!(moved.parent_id.unwrap(), child2.id);
        assert_eq!(moved.level_depth, 2);
    }

    #[tokio::test]
    async fn test_move_node_circular() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let root = e.add_node(h.id, org, None, "0000", None, "root", 1).await.unwrap();
        let child = e.add_node(h.id, org, Some(root.id), "1000", None, "summary", 1).await.unwrap();
        let grandchild = e.add_node(h.id, org, Some(child.id), "1100", None, "detail", 1).await.unwrap();
        // Cannot move child under grandchild (circular)
        assert!(e.move_node(child.id, Some(grandchild.id), 1).await.is_err());
    }

    #[tokio::test]
    async fn test_get_ancestors() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let root = e.add_node(h.id, org, None, "0000", Some("All"), "root", 1).await.unwrap();
        let child = e.add_node(h.id, org, Some(root.id), "1000", Some("Assets"), "summary", 1).await.unwrap();
        let grandchild = e.add_node(h.id, org, Some(child.id), "1100", Some("Cash"), "detail", 1).await.unwrap();
        let ancestors = e.get_ancestors(grandchild.id).await.unwrap();
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0].account_code, "1000");
        assert_eq!(ancestors[1].account_code, "0000");
    }

    #[tokio::test]
    async fn test_get_descendants() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let root = e.add_node(h.id, org, None, "0000", Some("All"), "root", 1).await.unwrap();
        let _ = e.add_node(h.id, org, Some(root.id), "1000", Some("Assets"), "summary", 1).await.unwrap();
        let _ = e.add_node(h.id, org, Some(root.id), "2000", Some("Liabilities"), "summary", 2).await.unwrap();
        let descendants = e.get_descendants(root.id).await.unwrap();
        assert_eq!(descendants.len(), 2);
    }

    #[tokio::test]
    async fn test_list_hierarchies_invalid_type() {
        assert!(eng().list_hierarchies(Uuid::new_v4(), Some("invalid"), None).await.is_err());
    }

    #[tokio::test]
    async fn test_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart", None, "account", None, None, None).await.unwrap();
        let root = e.add_node(h.id, org, None, "0000", None, "root", 1).await.unwrap();
        let _ = e.add_node(h.id, org, Some(root.id), "1000", None, "summary", 1).await.unwrap();
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_hierarchies, 1);
        assert_eq!(dash.active_hierarchies, 1);
        assert_eq!(dash.total_nodes, 2);
    }

    #[tokio::test]
    async fn test_full_hierarchy_build() {
        let e = eng();
        let org = Uuid::new_v4();
        let h = e.create_hierarchy(org, "COA", "Chart of Accounts", Some("Full COA"), "account", None, None, None).await.unwrap();

        let root = e.add_node(h.id, org, None, "0000", Some("All Accounts"), "root", 1).await.unwrap();
        let assets = e.add_node(h.id, org, Some(root.id), "1000", Some("Assets"), "summary", 1).await.unwrap();
        let liabilities = e.add_node(h.id, org, Some(root.id), "2000", Some("Liabilities"), "summary", 2).await.unwrap();
        let cash = e.add_node(h.id, org, Some(assets.id), "1100", Some("Cash"), "detail", 1).await.unwrap();
        let bank = e.add_node(h.id, org, Some(assets.id), "1200", Some("Bank"), "detail", 2).await.unwrap();

        let all_nodes = e.list_nodes(h.id).await.unwrap();
        assert_eq!(all_nodes.len(), 5);

        let asset_children = e.list_children(assets.id).await.unwrap();
        assert_eq!(asset_children.len(), 2);

        let cash_ancestors = e.get_ancestors(cash.id).await.unwrap();
        assert_eq!(cash_ancestors.len(), 2);

        let root_descendants = e.get_descendants(root.id).await.unwrap();
        assert_eq!(root_descendants.len(), 4);

        // Move bank under liabilities
        let moved = e.move_node(bank.id, Some(liabilities.id), 1).await.unwrap();
        assert_eq!(moved.level_depth, 2);
        let liab_children = e.list_children(liabilities.id).await.unwrap();
        assert_eq!(liab_children.len(), 1);
        assert_eq!(liab_children[0].account_code, "1200");
    }
}
