//! Schema handlers

use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use atlas_shared::EntityDefinition;
use crate::AppState;
use std::sync::Arc;
use tracing::debug;

/// Get entity schema definition
pub async fn get_entity_schema(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
) -> Result<Json<EntityDefinition>, StatusCode> {
    debug!("Getting schema for entity: {}", entity);
    
    match state.schema_engine.get_entity(&entity) {
        Some(schema) => Ok(Json(schema)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormConfig {
    pub entity: String,
    pub fields: Vec<FieldConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldConfig {
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub visible: bool,
    pub editable: bool,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
    pub options: Option<Vec<String>>,
}

/// Get form configuration for an entity
pub async fn get_entity_form(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
) -> Result<Json<FormConfig>, StatusCode> {
    debug!("Getting form config for entity: {}", entity);
    
    let entity_def = match state.schema_engine.get_entity(&entity) {
        Some(def) => def,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    let fields: Vec<FieldConfig> = entity_def.fields.iter().map(|f| {
        let ft = match &f.field_type {
            atlas_shared::FieldType::Enum { values } => ("select".to_string(), Some(values.clone())),
            atlas_shared::FieldType::String { .. } => ("text".to_string(), None),
            atlas_shared::FieldType::Integer { .. } => ("number".to_string(), None),
            atlas_shared::FieldType::Decimal { .. } => ("decimal".to_string(), None),
            atlas_shared::FieldType::Boolean => ("checkbox".to_string(), None),
            atlas_shared::FieldType::Date => ("date".to_string(), None),
            atlas_shared::FieldType::DateTime => ("datetime".to_string(), None),
            _ => ("text".to_string(), None),
        };
        
        FieldConfig {
            name: f.name.clone(),
            label: f.label.clone(),
            field_type: ft.0,
            required: f.is_required,
            visible: true,
            editable: !f.is_read_only,
            placeholder: f.placeholder.clone(),
            help_text: f.help_text.clone(),
            options: ft.1,
        }
    }).collect();
    
    Ok(Json(FormConfig {
        entity,
        fields,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListViewConfig {
    pub entity: String,
    pub columns: Vec<ColumnConfig>,
    pub default_sort: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnConfig {
    pub field: String,
    pub label: String,
    pub width: Option<u32>,
    pub sortable: bool,
}

/// Get list view configuration for an entity
pub async fn get_entity_list_view(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
) -> Result<Json<ListViewConfig>, StatusCode> {
    debug!("Getting list view config for entity: {}", entity);
    
    let entity_def = match state.schema_engine.get_entity(&entity) {
        Some(def) => def,
        None => return Err(StatusCode::NOT_FOUND),
    };
    
    let columns: Vec<ColumnConfig> = entity_def.fields.iter()
        .filter(|f| f.is_searchable)
        .map(|f| ColumnConfig {
            field: f.name.clone(),
            label: f.label.clone(),
            width: None,
            sortable: true,
        })
        .collect();
    
    Ok(Json(ListViewConfig {
        entity,
        columns,
        default_sort: Some("created_at".to_string()),
    }))
}
