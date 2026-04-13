//! Schema Engine
//! 
//! The schema engine provides dynamic entity definitions at runtime.
//! Entities are stored in the database and can be modified without restarts.

mod engine;
mod repository;
mod builder;
mod query;

pub use engine::SchemaEngine;
pub use repository::{SchemaRepository, PostgresSchemaRepository};
pub use builder::{SchemaBuilder, WorkflowBuilder};
pub use query::DynamicQuery;

use atlas_shared::{EntityDefinition, FieldDefinition, FieldType};
use std::collections::HashMap;

/// Cached entity definition
#[derive(Debug, Clone)]
pub struct CachedEntity {
    pub definition: EntityDefinition,
    pub field_map: HashMap<String, FieldDefinition>,
    pub version: i64,
}

impl CachedEntity {
    pub fn new(definition: EntityDefinition, version: i64) -> Self {
        let field_map: HashMap<_, _> = definition.fields.iter()
            .map(|f| (f.name.clone(), f.clone()))
            .collect();
        
        Self { definition, field_map, version }
    }
    
    pub fn get_field(&self, name: &str) -> Option<&FieldDefinition> {
        self.field_map.get(name)
    }
}

/// SQL type mapping for field types
pub fn field_type_to_sql(field_type: &FieldType) -> &'static str {
    match field_type {
        FieldType::String { .. } => "TEXT",
        FieldType::FixedString { .. } => "VARCHAR",
        FieldType::Integer { .. } => "BIGINT",
        FieldType::Decimal { .. } => "NUMERIC",
        FieldType::Boolean => "BOOLEAN",
        FieldType::Date => "DATE",
        FieldType::DateTime => "TIMESTAMPTZ",
        FieldType::Enum { .. } => "VARCHAR(100)",
        FieldType::Currency { .. } => "NUMERIC(18,2)",
        FieldType::Json => "JSONB",
        FieldType::Email => "VARCHAR(255)",
        FieldType::Url => "VARCHAR(2048)",
        FieldType::Phone => "VARCHAR(50)",
        FieldType::RichText => "TEXT",
        FieldType::Attachment => "UUID",
        FieldType::Reference { .. } => "UUID",
        FieldType::OneToMany { .. } => panic!("OneToMany is not a SQL column type"),
        FieldType::OneToOne { .. } => "UUID",
        FieldType::Computed { .. } => panic!("Computed fields have no SQL column"),
        FieldType::Address => "JSONB",
    }
}

/// Generate CREATE TABLE SQL for an entity
pub fn generate_create_table_sql(entity: &EntityDefinition) -> String {
    let table_name = entity.table_name.as_deref().unwrap_or(&entity.name);
    let mut columns = vec![
        "id UUID PRIMARY KEY DEFAULT gen_random_uuid()".to_string(),
        "organization_id UUID".to_string(),
        "created_at TIMESTAMPTZ DEFAULT now()".to_string(),
        "updated_at TIMESTAMPTZ DEFAULT now()".to_string(),
        "created_by UUID".to_string(),
        "updated_by UUID".to_string(),
    ];
    
    if entity.is_soft_delete {
        columns.push("deleted_at TIMESTAMPTZ".to_string());
    }
    
    for field in &entity.fields {
        let col_type = field_type_to_sql(&field.field_type);
        
        let mut col_def = format!("\"{}\" {}", field.name, col_type);
        
        if field.is_required {
            col_def.push_str(" NOT NULL");
        }
        
        if let Some(default) = &field.default_value {
            col_def.push_str(&format!(" DEFAULT {}", default));
        }
        
        columns.push(col_def);
    }
    
    // Add standard indexes
    columns.push("UNIQUE(organization_id, id)".to_string());
    
    format!(
        "CREATE TABLE IF NOT EXISTS \"{}\" (\n  {}\n);",
        table_name,
        columns.join(",\n  ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_shared::{EntityDefinition, FieldDefinition, FieldType};
    use uuid::Uuid;
    
    fn create_test_entity() -> EntityDefinition {
        EntityDefinition {
            id: Some(Uuid::new_v4()),
            name: "test_entity".to_string(),
            label: "Test Entity".to_string(),
            plural_label: "Test Entities".to_string(),
            table_name: Some("test_entities".to_string()),
            description: None,
            fields: vec![
                FieldDefinition::new("name", "Name", FieldType::String { max_length: Some(100), pattern: None }),
                FieldDefinition::new("amount", "Amount", FieldType::Decimal { precision: 12, scale: 2 }),
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
    fn test_generate_create_table() {
        let entity = create_test_entity();
        let sql = generate_create_table_sql(&entity);
        
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS \"test_entities\""));
        assert!(sql.contains("\"name\" TEXT"));
        assert!(sql.contains("\"amount\" NUMERIC"));
        assert!(sql.contains("\"status\" VARCHAR(100)"));
        assert!(sql.contains("deleted_at TIMESTAMPTZ"));
    }
    
    #[test]
    fn test_field_type_to_sql() {
        assert_eq!(field_type_to_sql(&FieldType::String { max_length: None, pattern: None }), "TEXT");
        assert_eq!(field_type_to_sql(&FieldType::Integer { min: None, max: None }), "BIGINT");
        assert_eq!(field_type_to_sql(&FieldType::Boolean), "BOOLEAN");
        assert_eq!(field_type_to_sql(&FieldType::Date), "DATE");
        assert_eq!(field_type_to_sql(&FieldType::DateTime), "TIMESTAMPTZ");
        assert_eq!(field_type_to_sql(&FieldType::Json), "JSONB");
    }
}
