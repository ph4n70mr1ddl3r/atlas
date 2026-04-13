//! Schema Repository
//! 
//! Trait and implementations for storing entity definitions.

use atlas_shared::{EntityDefinition, AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::FromRow;
use uuid::Uuid;
use serde_json::Value;

/// Repository trait for entity definitions
#[async_trait]
pub trait SchemaRepository: Send + Sync {
    /// Get all entity definitions
    async fn get_all_entities(&self) -> AtlasResult<Vec<EntityDefinition>>;
    
    /// Get a single entity by name
    async fn get_entity(&self, name: &str) -> AtlasResult<Option<EntityDefinition>>;
    
    /// Create or update an entity
    async fn upsert_entity(&self, entity: &EntityDefinition) -> AtlasResult<()>;
    
    /// Delete an entity
    async fn delete_entity(&self, name: &str) -> AtlasResult<()>;
    
    /// Get entity version
    async fn get_entity_version(&self, name: &str) -> AtlasResult<Option<i64>>;
    
    /// Set entity version
    async fn set_entity_version(&self, name: &str, version: i64) -> AtlasResult<()>;
}

#[derive(FromRow)]
struct EntityRow {
    id: Option<Uuid>,
    name: String,
    label: String,
    plural_label: String,
    table_name: Option<String>,
    description: Option<String>,
    fields: Value,
    indexes: Value,
    workflow: Option<Value>,
    security: Option<Value>,
    is_audit_enabled: bool,
    is_soft_delete: bool,
    icon: Option<String>,
    color: Option<String>,
    metadata: Value,
}

/// PostgreSQL implementation of SchemaRepository
pub struct PostgresSchemaRepository {
    pool: PgPool,
}

impl PostgresSchemaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SchemaRepository for PostgresSchemaRepository {
    async fn get_all_entities(&self) -> AtlasResult<Vec<EntityDefinition>> {
        let rows = sqlx::query_as::<_, EntityRow>(
            r#"
            SELECT 
                id, name, label, plural_label, table_name, description,
                fields, indexes, workflow, security,
                is_audit_enabled, is_soft_delete, icon, color, metadata
            FROM _atlas.entities
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    async fn get_entity(&self, name: &str) -> AtlasResult<Option<EntityDefinition>> {
        let row = sqlx::query_as::<_, EntityRow>(
            r#"
            SELECT 
                id, name, label, plural_label, table_name, description,
                fields, indexes, workflow, security,
                is_audit_enabled, is_soft_delete, icon, color, metadata
            FROM _atlas.entities
            WHERE name = $1
            "#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| r.into()))
    }
    
    async fn upsert_entity(&self, entity: &EntityDefinition) -> AtlasResult<()> {
        let id = entity.id.unwrap_or_else(Uuid::new_v4);
        let fields_json = serde_json::to_value(&entity.fields).map_err(|e| AtlasError::SchemaError(e.to_string()))?;
        let indexes_json = serde_json::to_value(&entity.indexes).map_err(|e| AtlasError::SchemaError(e.to_string()))?;
        let workflow_json = entity.workflow.as_ref().and_then(|w| serde_json::to_value(w).ok());
        let security_json = entity.security.as_ref().and_then(|s| serde_json::to_value(s).ok());
        
        sqlx::query(
            r#"
            INSERT INTO _atlas.entities (
                id, name, label, plural_label, table_name, description,
                fields, indexes, workflow, security,
                is_audit_enabled, is_soft_delete, icon, color, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (name) DO UPDATE SET
                label = EXCLUDED.label,
                plural_label = EXCLUDED.plural_label,
                table_name = EXCLUDED.table_name,
                description = EXCLUDED.description,
                fields = EXCLUDED.fields,
                indexes = EXCLUDED.indexes,
                workflow = EXCLUDED.workflow,
                security = EXCLUDED.security,
                is_audit_enabled = EXCLUDED.is_audit_enabled,
                is_soft_delete = EXCLUDED.is_soft_delete,
                icon = EXCLUDED.icon,
                color = EXCLUDED.color,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            "#
        )
        .bind(id)
        .bind(&entity.name)
        .bind(&entity.label)
        .bind(&entity.plural_label)
        .bind(&entity.table_name)
        .bind(&entity.description)
        .bind(&fields_json)
        .bind(&indexes_json)
        .bind(&workflow_json)
        .bind(&security_json)
        .bind(entity.is_audit_enabled)
        .bind(entity.is_soft_delete)
        .bind(&entity.icon)
        .bind(&entity.color)
        .bind(&entity.metadata)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn delete_entity(&self, name: &str) -> AtlasResult<()> {
        sqlx::query("DELETE FROM _atlas.entities WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    async fn get_entity_version(&self, name: &str) -> AtlasResult<Option<i64>> {
        let row = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT version FROM _atlas.config_versions 
            WHERE entity_name = $1
            ORDER BY version DESC LIMIT 1
            "#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row)
    }
    
    async fn set_entity_version(&self, name: &str, version: i64) -> AtlasResult<()> {
        let config = serde_json::json!({});
        
        sqlx::query(
            r#"
            INSERT INTO _atlas.config_versions (entity_name, version, config)
            VALUES ($1, $2, $3)
            "#
        )
        .bind(name)
        .bind(version)
        .bind(&config)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}

impl From<EntityRow> for EntityDefinition {
    fn from(row: EntityRow) -> Self {
        EntityDefinition {
            id: row.id,
            name: row.name,
            label: row.label,
            plural_label: row.plural_label,
            table_name: row.table_name,
            description: row.description,
            fields: serde_json::from_value(row.fields).unwrap_or_default(),
            indexes: serde_json::from_value(row.indexes).unwrap_or_default(),
            workflow: row.workflow.and_then(|w| serde_json::from_value(w).ok()),
            security: row.security.and_then(|s| serde_json::from_value(s).ok()),
            is_audit_enabled: row.is_audit_enabled,
            is_soft_delete: row.is_soft_delete,
            icon: row.icon,
            color: row.color,
            metadata: row.metadata,
        }
    }
}
