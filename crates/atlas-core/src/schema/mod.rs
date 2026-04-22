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
pub fn field_type_to_sql(field_type: &FieldType) -> String {
    match field_type {
        FieldType::String { .. } => "TEXT".to_string(),
        FieldType::FixedString { length } => format!("VARCHAR({})", length),
        FieldType::Integer { .. } => "BIGINT".to_string(),
        FieldType::Decimal { precision, scale } => format!("NUMERIC({}, {})", precision, scale),
        FieldType::Boolean => "BOOLEAN".to_string(),
        FieldType::Date => "DATE".to_string(),
        FieldType::DateTime => "TIMESTAMPTZ".to_string(),
        FieldType::Enum { .. } => "VARCHAR(100)".to_string(),
        FieldType::Currency { .. } => "NUMERIC(18,2)".to_string(),
        FieldType::Json => "JSONB".to_string(),
        FieldType::Email => "VARCHAR(255)".to_string(),
        FieldType::Url => "VARCHAR(2048)".to_string(),
        FieldType::Phone => "VARCHAR(50)".to_string(),
        FieldType::RichText => "TEXT".to_string(),
        FieldType::Attachment => "UUID".to_string(),
        FieldType::Reference { .. } => "UUID".to_string(),
        FieldType::OneToMany { .. } => "TEXT".to_string(), // Stored as JSON array of IDs; not a direct column
        FieldType::OneToOne { .. } => "UUID".to_string(),
        FieldType::Computed { .. } => "TEXT".to_string(), // Virtual column; may be materialized or omitted
        FieldType::Address => "JSONB".to_string(),
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
        let safe_name = field.name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>();
        
        let mut col_def = format!("\"{}\" {}", safe_name, col_type);
        
        if field.is_required {
            col_def.push_str(" NOT NULL");
        }
        
        if let Some(default) = &field.default_value {
            // Safely serialize the default value for SQL
            let default_str = match default {
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::String(s) => {
                    let escaped = s.replace('\'', "''");
                    format!("'{}'", escaped)
                }
                _ => {
                    // For complex types, use quoted JSON
                    let escaped = default.to_string().replace('\'', "''");
                    format!("'{}'", escaped)
                }
            };
            col_def.push_str(&format!(" DEFAULT {}", default_str));
        }
        
        columns.push(col_def);
    }
    
    // Add standard indexes
    columns.push("UNIQUE(organization_id, id)".to_string());
    
    // Sanitize table name: reject embedded quotes or semicolons to prevent injection
    if table_name.contains('"') || table_name.contains(';') || table_name.contains("--") {
        return "-- ERROR: invalid table name".to_string();
    }
    
    format!(
        "CREATE TABLE IF NOT EXISTS \"{}\" (\n  {}\n);",
        table_name,
        columns.join(",\n  ")
    )
}

/// Generate CREATE INDEX SQL statements for an entity
pub fn generate_index_sql(entity: &EntityDefinition) -> Vec<String> {
    let table_name = entity.table_name.as_deref().unwrap_or(&entity.name);
    entity.indexes.iter().map(|idx| {
        let safe_idx_name: String = idx.name.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let fields: Vec<String> = idx.fields.iter()
            .map(|f| {
                let safe_f: String = f.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect();
                format!("\"{}\"", safe_f)
            })
            .collect();
        let fields_str = fields.join(", ");
        let unique = if idx.is_unique { "UNIQUE " } else { "" };
        format!(
            "CREATE{} INDEX IF NOT EXISTS \"{}\" ON \"{}\" ({})",
            unique, safe_idx_name, table_name, fields_str
        )
    }).collect()
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
