//! Financial Dimension Set Module
//!
//! Oracle Fusion Cloud ERP-inspired Financial Dimension Sets.
//! Groups financial dimensions for reporting, analysis, and allocation purposes.
//!
//! Oracle Fusion equivalent: Financials > General Ledger > Financial Dimension Sets

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinancialDimensionSet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub members: Vec<DimensionSetMember>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionSetMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub dimension_set_id: Uuid,
    pub dimension_id: Uuid,
    pub dimension_code: String,
    pub dimension_value_id: Uuid,
    pub dimension_value_code: String,
    pub display_order: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionSetDashboard {
    pub organization_id: Uuid,
    pub total_sets: i32,
    pub active_sets: i32,
    pub by_member_count: serde_json::Value,
}

// ============================================================================
// Repository
// ============================================================================

#[async_trait]
pub trait FinancialDimensionSetRepository: Send + Sync {
    async fn create_set(&self,
        org_id: Uuid, code: &str, name: &str, description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialDimensionSet>;

    async fn get(&self, id: Uuid) -> AtlasResult<Option<FinancialDimensionSet>>;
    async fn get_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FinancialDimensionSet>>;
    async fn list(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<FinancialDimensionSet>>;
    async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>) -> AtlasResult<FinancialDimensionSet>;
    async fn deactivate(&self, id: Uuid) -> AtlasResult<FinancialDimensionSet>;
    async fn activate(&self, id: Uuid) -> AtlasResult<FinancialDimensionSet>;
    async fn delete_set(&self, org_id: Uuid, code: &str) -> AtlasResult<()>;

    async fn add_member(&self,
        org_id: Uuid, dimension_set_id: Uuid,
        dimension_id: Uuid, dimension_code: &str,
        dimension_value_id: Uuid, dimension_value_code: &str,
        display_order: i32,
    ) -> AtlasResult<DimensionSetMember>;

    async fn list_members(&self, dimension_set_id: Uuid) -> AtlasResult<Vec<DimensionSetMember>>;
    async fn remove_member(&self, id: Uuid) -> AtlasResult<()>;

    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DimensionSetDashboard>;
}

/// PostgreSQL stub implementation
#[allow(dead_code)]
pub struct PostgresFinancialDimensionSetRepository { #[allow(dead_code)]
    pool: PgPool }
impl PostgresFinancialDimensionSetRepository { pub fn new(pool: PgPool) -> Self { Self { pool } } }

#[async_trait]
impl FinancialDimensionSetRepository for PostgresFinancialDimensionSetRepository {
    async fn create_set(&self, _: Uuid, _: &str, _: &str, _: Option<&str>, _: Option<Uuid>) -> AtlasResult<FinancialDimensionSet> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn get(&self, _: Uuid) -> AtlasResult<Option<FinancialDimensionSet>> { Ok(None) }
    async fn get_by_code(&self, _: Uuid, _: &str) -> AtlasResult<Option<FinancialDimensionSet>> { Ok(None) }
    async fn list(&self, _: Uuid, _: Option<bool>) -> AtlasResult<Vec<FinancialDimensionSet>> { Ok(vec![]) }
    async fn update(&self, _: Uuid, _: Option<&str>, _: Option<&str>) -> AtlasResult<FinancialDimensionSet> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn deactivate(&self, _: Uuid) -> AtlasResult<FinancialDimensionSet> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn activate(&self, _: Uuid) -> AtlasResult<FinancialDimensionSet> { Err(AtlasError::EntityNotFound("Not found".into())) }
    async fn delete_set(&self, _: Uuid, _: &str) -> AtlasResult<()> { Ok(()) }
    async fn add_member(&self, _: Uuid, _: Uuid, _: Uuid, _: &str, _: Uuid, _: &str, _: i32) -> AtlasResult<DimensionSetMember> { Err(AtlasError::DatabaseError("Not implemented".into())) }
    async fn list_members(&self, _: Uuid) -> AtlasResult<Vec<DimensionSetMember>> { Ok(vec![]) }
    async fn remove_member(&self, _: Uuid) -> AtlasResult<()> { Ok(()) }
    async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DimensionSetDashboard> {
        Ok(DimensionSetDashboard {
            organization_id: org_id, total_sets: 0, active_sets: 0, by_member_count: serde_json::json!([]),
        })
    }
}

// ============================================================================
// Engine
// ============================================================================

use std::sync::Arc;
use tracing::info;

pub struct FinancialDimensionSetEngine {
    repository: Arc<dyn FinancialDimensionSetRepository>,
}

impl FinancialDimensionSetEngine {
    pub fn new(repository: Arc<dyn FinancialDimensionSetRepository>) -> Self {
        Self { repository }
    }

    /// Create a new dimension set
    pub async fn create(
        &self,
        org_id: Uuid,
        code: &str,
        name: &str,
        description: Option<&str>,
        created_by: Option<Uuid>,
    ) -> AtlasResult<FinancialDimensionSet> {
        if code.is_empty() || name.is_empty() {
            return Err(AtlasError::ValidationFailed("Code and name are required".into()));
        }
        if self.repository.get_by_code(org_id, code).await?.is_some() {
            return Err(AtlasError::Conflict(format!("Dimension set '{}' already exists", code)));
        }
        info!("Creating financial dimension set '{}' for org {}", code, org_id);
        self.repository.create_set(org_id, code, name, description, created_by).await
    }

    /// Get by ID
    pub async fn get(&self, id: Uuid) -> AtlasResult<Option<FinancialDimensionSet>> {
        self.repository.get(id).await
    }

    /// Get by code
    pub async fn get_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FinancialDimensionSet>> {
        self.repository.get_by_code(org_id, code).await
    }

    /// List dimension sets
    pub async fn list(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<FinancialDimensionSet>> {
        self.repository.list(org_id, is_active).await
    }

    /// Update dimension set
    pub async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>) -> AtlasResult<FinancialDimensionSet> {
        let ds = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Dimension set {} not found", id)))?;
        if !ds.is_active {
            return Err(AtlasError::ValidationFailed("Cannot update inactive dimension set".into()));
        }
        info!("Updating dimension set {}", ds.code);
        self.repository.update(id, name, description).await
    }

    /// Deactivate
    pub async fn deactivate(&self, id: Uuid) -> AtlasResult<FinancialDimensionSet> {
        let ds = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Dimension set {} not found", id)))?;
        if !ds.is_active { return Err(AtlasError::ValidationFailed("Already inactive".into())); }
        info!("Deactivating dimension set {}", ds.code);
        self.repository.deactivate(id).await
    }

    /// Activate
    pub async fn activate(&self, id: Uuid) -> AtlasResult<FinancialDimensionSet> {
        let ds = self.repository.get(id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Dimension set {} not found", id)))?;
        if ds.is_active { return Err(AtlasError::ValidationFailed("Already active".into())); }
        info!("Activating dimension set {}", ds.code);
        self.repository.activate(id).await
    }

    /// Delete
    pub async fn delete(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
        self.repository.get_by_code(org_id, code).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Dimension set '{}' not found", code)))?;
        info!("Deleting dimension set {}", code);
        self.repository.delete_set(org_id, code).await
    }

    /// Add a member to a dimension set
    pub async fn add_member(
        &self,
        org_id: Uuid,
        dimension_set_id: Uuid,
        dimension_id: Uuid,
        dimension_code: &str,
        dimension_value_id: Uuid,
        dimension_value_code: &str,
        display_order: i32,
    ) -> AtlasResult<DimensionSetMember> {
        let ds = self.repository.get(dimension_set_id).await?
            .ok_or_else(|| AtlasError::EntityNotFound(format!("Dimension set {} not found", dimension_set_id)))?;

        if !ds.is_active {
            return Err(AtlasError::ValidationFailed("Cannot add members to inactive dimension set".into()));
        }
        if dimension_code.is_empty() || dimension_value_code.is_empty() {
            return Err(AtlasError::ValidationFailed("Dimension code and value code are required".into()));
        }

        // Check for duplicate member
        let members = self.repository.list_members(dimension_set_id).await?;
        if members.iter().any(|m| m.dimension_code == dimension_code && m.dimension_value_code == dimension_value_code) {
            return Err(AtlasError::Conflict(format!(
                "Member {}.{} already exists in dimension set", dimension_code, dimension_value_code
            )));
        }

        info!("Adding member {}.{} to dimension set {}", dimension_code, dimension_value_code, ds.code);
        self.repository.add_member(
            org_id, dimension_set_id,
            dimension_id, dimension_code,
            dimension_value_id, dimension_value_code,
            display_order,
        ).await
    }

    /// List members
    pub async fn list_members(&self, dimension_set_id: Uuid) -> AtlasResult<Vec<DimensionSetMember>> {
        self.repository.list_members(dimension_set_id).await
    }

    /// Remove a member
    pub async fn remove_member(&self, id: Uuid) -> AtlasResult<()> {
        self.repository.remove_member(id).await
    }

    /// Get dashboard
    pub async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DimensionSetDashboard> {
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
        sets: std::sync::Mutex<Vec<FinancialDimensionSet>>,
        members: std::sync::Mutex<Vec<DimensionSetMember>>,
    }
    impl MockRepo { fn new() -> Self { Self { sets: std::sync::Mutex::new(vec![]), members: std::sync::Mutex::new(vec![]) } } }

    #[async_trait]
    impl FinancialDimensionSetRepository for MockRepo {
        async fn create_set(&self, org_id: Uuid, code: &str, name: &str, description: Option<&str>, created_by: Option<Uuid>) -> AtlasResult<FinancialDimensionSet> {
            let ds = FinancialDimensionSet {
                id: Uuid::new_v4(), organization_id: org_id, code: code.into(), name: name.into(),
                description: description.map(Into::into), members: vec![], is_active: true,
                metadata: serde_json::json!({}), created_by, created_at: Utc::now(), updated_at: Utc::now(),
            };
            self.sets.lock().unwrap().push(ds.clone());
            Ok(ds)
        }
        async fn get(&self, id: Uuid) -> AtlasResult<Option<FinancialDimensionSet>> {
            let mut sets = self.sets.lock().unwrap();
            if let Some(ds) = sets.iter_mut().find(|s| s.id == id) {
                ds.members = self.members.lock().unwrap().iter().filter(|m| m.dimension_set_id == id).cloned().collect();
                Ok(Some(ds.clone()))
            } else { Ok(None) }
        }
        async fn get_by_code(&self, org_id: Uuid, code: &str) -> AtlasResult<Option<FinancialDimensionSet>> {
            Ok(self.sets.lock().unwrap().iter().find(|s| s.organization_id == org_id && s.code == code).cloned())
        }
        async fn list(&self, org_id: Uuid, is_active: Option<bool>) -> AtlasResult<Vec<FinancialDimensionSet>> {
            Ok(self.sets.lock().unwrap().iter()
                .filter(|s| s.organization_id == org_id && (is_active.is_none() || s.is_active == is_active.unwrap()))
                .cloned().collect())
        }
        async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>) -> AtlasResult<FinancialDimensionSet> {
            let mut sets = self.sets.lock().unwrap();
            let ds = sets.iter_mut().find(|s| s.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            if let Some(n) = name { ds.name = n.into(); }
            if let Some(d) = description { ds.description = Some(d.into()); }
            ds.updated_at = Utc::now();
            Ok(ds.clone())
        }
        async fn deactivate(&self, id: Uuid) -> AtlasResult<FinancialDimensionSet> {
            let mut sets = self.sets.lock().unwrap();
            let ds = sets.iter_mut().find(|s| s.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ds.is_active = false; ds.updated_at = Utc::now(); Ok(ds.clone())
        }
        async fn activate(&self, id: Uuid) -> AtlasResult<FinancialDimensionSet> {
            let mut sets = self.sets.lock().unwrap();
            let ds = sets.iter_mut().find(|s| s.id == id).ok_or_else(|| AtlasError::EntityNotFound("Not found".into()))?;
            ds.is_active = true; ds.updated_at = Utc::now(); Ok(ds.clone())
        }
        async fn delete_set(&self, org_id: Uuid, code: &str) -> AtlasResult<()> {
            self.sets.lock().unwrap().retain(|s| !(s.organization_id == org_id && s.code == code));
            Ok(())
        }
        async fn add_member(&self, org_id: Uuid, dimension_set_id: Uuid, dimension_id: Uuid, dimension_code: &str, dimension_value_id: Uuid, dimension_value_code: &str, display_order: i32) -> AtlasResult<DimensionSetMember> {
            let m = DimensionSetMember {
                id: Uuid::new_v4(), organization_id: org_id, dimension_set_id,
                dimension_id, dimension_code: dimension_code.into(),
                dimension_value_id, dimension_value_code: dimension_value_code.into(),
                display_order, metadata: serde_json::json!({}), created_at: Utc::now(),
            };
            self.members.lock().unwrap().push(m.clone());
            Ok(m)
        }
        async fn list_members(&self, dimension_set_id: Uuid) -> AtlasResult<Vec<DimensionSetMember>> {
            Ok(self.members.lock().unwrap().iter().filter(|m| m.dimension_set_id == dimension_set_id).cloned().collect())
        }
        async fn remove_member(&self, id: Uuid) -> AtlasResult<()> {
            self.members.lock().unwrap().retain(|m| m.id != id);
            Ok(())
        }
        async fn get_dashboard(&self, org_id: Uuid) -> AtlasResult<DimensionSetDashboard> {
            let sets = self.sets.lock().unwrap();
            let org_sets: Vec<_> = sets.iter().filter(|s| s.organization_id == org_id).collect();
            Ok(DimensionSetDashboard {
                organization_id: org_id,
                total_sets: org_sets.len() as i32,
                active_sets: org_sets.iter().filter(|s| s.is_active).count() as i32,
                by_member_count: serde_json::json!([]),
            })
        }
    }

    fn eng() -> FinancialDimensionSetEngine { FinancialDimensionSetEngine::new(Arc::new(MockRepo::new())) }

    #[tokio::test]
    async fn test_create_valid() {
        let ds = eng().create(Uuid::new_v4(), "COST_CTR", "Cost Centers", Some("All cost centers"), None).await.unwrap();
        assert_eq!(ds.code, "COST_CTR");
        assert!(ds.is_active);
        assert!(ds.members.is_empty());
    }

    #[tokio::test]
    async fn test_create_empty_code() {
        assert!(eng().create(Uuid::new_v4(), "", "Name", None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_empty_name() {
        assert!(eng().create(Uuid::new_v4(), "CODE", "", None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_duplicate() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create(org, "DUP", "First", None, None).await.unwrap();
        assert!(e.create(org, "DUP", "Second", None, None).await.is_err());
    }

    #[tokio::test]
    async fn test_add_member() {
        let e = eng();
        let org = Uuid::new_v4();
        let ds = e.create(org, "DEPT", "Departments", None, None).await.unwrap();
        let member = e.add_member(org, ds.id, Uuid::new_v4(), "DEPARTMENT", Uuid::new_v4(), "IT", 1).await.unwrap();
        assert_eq!(member.dimension_code, "DEPARTMENT");
        assert_eq!(member.dimension_value_code, "IT");

        let members = e.list_members(ds.id).await.unwrap();
        assert_eq!(members.len(), 1);
    }

    #[tokio::test]
    async fn test_add_member_empty_codes() {
        let e = eng();
        let org = Uuid::new_v4();
        let ds = e.create(org, "TEST", "Test", None, None).await.unwrap();
        assert!(e.add_member(org, ds.id, Uuid::new_v4(), "", Uuid::new_v4(), "IT", 1).await.is_err());
        assert!(e.add_member(org, ds.id, Uuid::new_v4(), "DEPT", Uuid::new_v4(), "", 1).await.is_err());
    }

    #[tokio::test]
    async fn test_add_duplicate_member() {
        let e = eng();
        let org = Uuid::new_v4();
        let ds = e.create(org, "TEST", "Test", None, None).await.unwrap();
        let dim_id = Uuid::new_v4();
        let val_id = Uuid::new_v4();
        let _ = e.add_member(org, ds.id, dim_id, "DEPT", val_id, "IT", 1).await.unwrap();
        assert!(e.add_member(org, ds.id, dim_id, "DEPT", val_id, "IT", 2).await.is_err());
    }

    #[tokio::test]
    async fn test_remove_member() {
        let e = eng();
        let org = Uuid::new_v4();
        let ds = e.create(org, "TEST", "Test", None, None).await.unwrap();
        let m = e.add_member(org, ds.id, Uuid::new_v4(), "DEPT", Uuid::new_v4(), "HR", 1).await.unwrap();
        e.remove_member(m.id).await.unwrap();
        let members = e.list_members(ds.id).await.unwrap();
        assert!(members.is_empty());
    }

    #[tokio::test]
    async fn test_deactivate_activate() {
        let e = eng();
        let org = Uuid::new_v4();
        let ds = e.create(org, "TEST", "Test", None, None).await.unwrap();
        let deactivated = e.deactivate(ds.id).await.unwrap();
        assert!(!deactivated.is_active);
        let activated = e.activate(ds.id).await.unwrap();
        assert!(activated.is_active);
    }

    #[tokio::test]
    async fn test_deactivate_already_inactive() {
        let e = eng();
        let org = Uuid::new_v4();
        let ds = e.create(org, "TEST", "Test", None, None).await.unwrap();
        let _ = e.deactivate(ds.id).await.unwrap();
        assert!(e.deactivate(ds.id).await.is_err());
    }

    #[tokio::test]
    async fn test_activate_already_active() {
        let e = eng();
        let org = Uuid::new_v4();
        let ds = e.create(org, "TEST", "Test", None, None).await.unwrap();
        assert!(e.activate(ds.id).await.is_err());
    }

    #[tokio::test]
    async fn test_add_member_to_inactive() {
        let e = eng();
        let org = Uuid::new_v4();
        let ds = e.create(org, "TEST", "Test", None, None).await.unwrap();
        let _ = e.deactivate(ds.id).await.unwrap();
        assert!(e.add_member(org, ds.id, Uuid::new_v4(), "DEPT", Uuid::new_v4(), "IT", 1).await.is_err());
    }

    #[tokio::test]
    async fn test_update() {
        let e = eng();
        let org = Uuid::new_v4();
        let ds = e.create(org, "TEST", "Test", None, None).await.unwrap();
        let updated = e.update(ds.id, Some("Updated"), Some("New desc")).await.unwrap();
        assert_eq!(updated.name, "Updated");
    }

    #[tokio::test]
    async fn test_update_inactive() {
        let e = eng();
        let org = Uuid::new_v4();
        let ds = e.create(org, "TEST", "Test", None, None).await.unwrap();
        let _ = e.deactivate(ds.id).await.unwrap();
        assert!(e.update(ds.id, Some("Updated"), None).await.is_err());
    }

    #[tokio::test]
    async fn test_delete() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create(org, "DEL", "To Delete", None, None).await.unwrap();
        e.delete(org, "DEL").await.unwrap();
        assert!(e.get_by_code(org, "DEL").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_dashboard() {
        let e = eng();
        let org = Uuid::new_v4();
        let _ = e.create(org, "DS1", "Set 1", None, None).await.unwrap();
        let _ = e.create(org, "DS2", "Set 2", None, None).await.unwrap();
        let dash = e.get_dashboard(org).await.unwrap();
        assert_eq!(dash.total_sets, 2);
        assert_eq!(dash.active_sets, 2);
    }
}
