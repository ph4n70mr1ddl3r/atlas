//! Schema Engine Implementation
//! 
//! Runtime schema management with caching and hot-reload support.

use atlas_shared::{EntityDefinition, FieldDefinition, AtlasError, AtlasResult};
use super::{CachedEntity, SchemaRepository};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Schema engine for dynamic entity management
pub struct SchemaEngine {
    repository: Arc<dyn SchemaRepository>,
    cache: Arc<DashMap<String, CachedEntity>>,
    version: Arc<RwLock<i64>>,
}

impl SchemaEngine {
    /// Create a new schema engine
    pub fn new(repository: Arc<dyn SchemaRepository>) -> Self {
        Self {
            repository,
            cache: Arc::new(DashMap::new()),
            version: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Load all entities from repository into cache
    pub async fn load_all(&self) -> AtlasResult<()> {
        info!("Loading all entity definitions into cache");
        
        let entities = self.repository.get_all_entities().await?;
        
        for entity in entities {
            let name = entity.name.clone();
            let version = self.repository.get_entity_version(&name).await?.unwrap_or(1);
            let cached = CachedEntity::new(entity, version);
            self.cache.insert(name, cached);
        }
        
        let count = self.cache.len();
        let max_version = self.cache.iter()
            .map(|e| e.value().version)
            .max()
            .unwrap_or(0);
        *self.version.write().await = max_version;
        
        info!("Loaded {} entities into schema cache", count);
        Ok(())
    }
    
    /// Get an entity definition by name
    pub fn get_entity(&self, name: &str) -> Option<EntityDefinition> {
        self.cache.get(name).map(|e| e.definition.clone())
    }
    
    /// Get a cached entity with version info
    pub fn get_cached_entity(&self, name: &str) -> Option<CachedEntity> {
        self.cache.get(name).map(|e| e.clone())
    }
    
    /// Get a specific field from an entity
    pub fn get_field(&self, entity_name: &str, field_name: &str) -> Option<FieldDefinition> {
        self.cache.get(entity_name)
            .and_then(|e| e.get_field(field_name).cloned())
    }
    
    /// Check if an entity exists
    pub fn has_entity(&self, name: &str) -> bool {
        self.cache.contains_key(name)
    }
    
    /// Get all entity names
    pub fn entity_names(&self) -> Vec<String> {
        self.cache.iter().map(|e| e.key().clone()).collect()
    }
    
    /// Get current schema version
    pub async fn get_version(&self) -> i64 {
        *self.version.read().await
    }
    
    /// Create or update an entity (hot-reload)
    pub async fn upsert_entity(&self, entity: EntityDefinition) -> AtlasResult<()> {
        let name = entity.name.clone();
        
        info!("Upserting entity: {}", name);
        
        // Validate entity
        self.validate_entity(&entity)?;
        
        // Get existing version or start at 1
        let existing_version = self.repository.get_entity_version(&name).await?;
        let new_version = existing_version.unwrap_or(0) + 1;
        
        // Save to repository
        self.repository.upsert_entity(&entity).await?;
        self.repository.set_entity_version(&name, new_version).await?;
        
        // Update cache
        let cached = CachedEntity::new(entity, new_version);
        self.cache.insert(name.clone(), cached);
        
        info!("Entity {} updated, version {}", name, new_version);
        
        // Publish schema changed event (handled by config system)
        Ok(())
    }
    
    /// Delete an entity
    pub async fn delete_entity(&self, name: &str) -> AtlasResult<()> {
        info!("Deleting entity: {}", name);
        
        if !self.cache.contains_key(name) {
            return Err(AtlasError::EntityNotFound(name.to_string()));
        }
        
        self.repository.delete_entity(name).await?;
        self.cache.remove(name);
        
        Ok(())
    }
    
    /// Validate entity definition
    fn validate_entity(&self, entity: &EntityDefinition) -> AtlasResult<()> {
        // Check for duplicate field names
        let mut field_names = std::collections::HashSet::new();
        for field in &entity.fields {
            if !field_names.insert(&field.name) {
                return Err(AtlasError::SchemaError(
                    format!("Duplicate field name: {}", field.name)
                ));
            }
        }
        
        // Check reserved field names
        let reserved = ["id", "organization_id", "created_at", "updated_at", 
                        "created_by", "updated_by", "deleted_at"];
        for field in &entity.fields {
            if reserved.contains(&field.name.as_str()) {
                return Err(AtlasError::SchemaError(
                    format!("Reserved field name: {}", field.name)
                ));
            }
        }
        
        // Validate workflow if present
        if let Some(workflow) = &entity.workflow {
            // Initial state must exist
            if !workflow.states.iter().any(|s| s.name == workflow.initial_state) {
                return Err(AtlasError::SchemaError(
                    "Initial state not found in workflow states".to_string()
                ));
            }
            
            // All transition from_states must exist
            for trans in &workflow.transitions {
                if !workflow.states.iter().any(|s| s.name == trans.from_state) {
                    return Err(AtlasError::SchemaError(
                        format!("Invalid from_state in transition: {}", trans.from_state)
                    ));
                }
                if !workflow.states.iter().any(|s| s.name == trans.to_state) {
                    return Err(AtlasError::SchemaError(
                        format!("Invalid to_state in transition: {}", trans.to_state)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Force refresh from database
    pub async fn refresh(&self) -> AtlasResult<()> {
        warn!("Refreshing schema cache from database");
        self.cache.clear();
        self.load_all().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_shared::{FieldDefinition, FieldType};
    use std::sync::Arc;
    
    struct MockRepository;
    
    #[async_trait::async_trait]
    impl SchemaRepository for MockRepository {
        async fn get_all_entities(&self) -> AtlasResult<Vec<EntityDefinition>> {
            Ok(vec![])
        }
        
        async fn get_entity(&self, _name: &str) -> AtlasResult<Option<EntityDefinition>> {
            Ok(None)
        }
        
        async fn upsert_entity(&self, _entity: &EntityDefinition) -> AtlasResult<()> {
            Ok(())
        }
        
        async fn delete_entity(&self, _name: &str) -> AtlasResult<()> {
            Ok(())
        }
        
        async fn get_entity_version(&self, _name: &str) -> AtlasResult<Option<i64>> {
            Ok(Some(1))
        }
        
        async fn set_entity_version(&self, _name: &str, _version: i64) -> AtlasResult<()> {
            Ok(())
        }
    }
    
    #[tokio::test]
    async fn test_schema_engine_basic() {
        let repo = Arc::new(MockRepository);
        let engine = SchemaEngine::new(repo);
        
        assert!(!engine.has_entity("test"));
    }
    
    #[tokio::test]
    async fn test_validate_entity_duplicate_fields() {
        let repo = Arc::new(MockRepository);
        let engine = SchemaEngine::new(repo);
        
        let entity = EntityDefinition {
            id: None,
            name: "test".to_string(),
            label: "Test".to_string(),
            plural_label: "Tests".to_string(),
            table_name: None,
            description: None,
            fields: vec![
                FieldDefinition::new("field1", "Field 1", FieldType::String { max_length: None, pattern: None }),
                FieldDefinition::new("field1", "Field 1 Duplicate", FieldType::Integer { min: None, max: None }),
            ],
            indexes: vec![],
            workflow: None,
            security: None,
            is_audit_enabled: true,
            is_soft_delete: true,
            icon: None,
            color: None,
            metadata: serde_json::Value::Null,
        };
        
        let result = engine.validate_entity(&entity);
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_validate_entity_reserved_field() {
        let repo = Arc::new(MockRepository);
        let engine = SchemaEngine::new(repo);
        
        let entity = EntityDefinition {
            id: None,
            name: "test".to_string(),
            label: "Test".to_string(),
            plural_label: "Tests".to_string(),
            table_name: None,
            description: None,
            fields: vec![
                FieldDefinition::new("id", "ID", FieldType::String { max_length: None, pattern: None }),
            ],
            indexes: vec![],
            workflow: None,
            security: None,
            is_audit_enabled: true,
            is_soft_delete: true,
            icon: None,
            color: None,
            metadata: serde_json::Value::Null,
        };
        
        let result = engine.validate_entity(&entity);
        assert!(result.is_err());
    }
}
