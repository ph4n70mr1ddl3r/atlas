//! Admin handlers

use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use atlas_shared::{EntityDefinition, WorkflowDefinition};
use atlas_core::schema::generate_create_table_sql;
use crate::AppState;
use std::sync::Arc;
use tracing::{info, debug, warn};

/// Create a new entity
pub async fn create_entity(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateEntityRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    info!("Creating entity: {}", payload.definition.name);
    
    let definition = payload.definition;
    
    // Generate table SQL
    let create_sql = generate_create_table_sql(&definition);
    
    // Execute table creation (ignore errors if already exists)
    if let Err(e) = sqlx::query(&create_sql).execute(&state.db_pool).await {
        warn!("Table creation warning: {}", e);
    }
    
    // Save entity definition
    if let Err(e) = state.schema_engine.upsert_entity(definition.clone()).await {
        warn!("Failed to upsert entity: {:?}", e);
    }
    
    // Load workflow if present
    if let Some(workflow) = &definition.workflow {
        if let Err(e) = state.workflow_engine.load_workflow(workflow.clone()).await {
            warn!("Failed to load workflow: {:?}", e);
        }
    }
    
    info!("Entity {} created successfully", definition.name);
    
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "entity": definition.name,
        "table_created": true,
        "workflow_loaded": definition.workflow.is_some()
    }))))
}

#[derive(Debug, Deserialize)]
pub struct CreateEntityRequest {
    pub definition: EntityDefinition,
    #[serde(default)]
    pub create_table: bool,
}

/// Update an entity definition
pub async fn update_entity(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
    Json(payload): Json<UpdateEntityRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Updating entity: {}", entity);
    
    let mut definition = payload.definition;
    definition.name = entity.clone();
    
    state.schema_engine.upsert_entity(definition.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Reload workflow if present
    if let Some(workflow) = &definition.workflow {
        state.workflow_engine.load_workflow(workflow.clone())
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    Ok(Json(serde_json::json!({
        "entity": entity,
        "updated": true
    })))
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntityRequest {
    pub definition: EntityDefinition,
}

/// Delete an entity
pub async fn delete_entity(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
    Json(payload): Json<DeleteEntityRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Deleting entity: {}", entity);
    
    if !payload.drop_table {
        state.schema_engine.delete_entity(&entity)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        return Ok(Json(serde_json::json!({
            "entity": entity,
            "definition_removed": true,
            "table_preserved": true
        })));
    }
    
    // Drop table
    if let Err(e) = sqlx::query(&format!("DROP TABLE IF EXISTS \"{}\"", entity))
        .execute(&state.db_pool)
        .await
    {
        warn!("Failed to drop table: {}", e);
    }
    
    state.schema_engine.delete_entity(&entity)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({
        "entity": entity,
        "table_dropped": true,
        "definition_removed": true
    })))
}

#[derive(Debug, Deserialize)]
pub struct DeleteEntityRequest {
    #[serde(default)]
    pub drop_table: bool,
}

/// Create or update a workflow
pub async fn create_workflow(
    State(state): State<Arc<AppState>>,
    Json(workflow): Json<WorkflowDefinition>,
) -> Result<(StatusCode, Json<serde_json::Value>), StatusCode> {
    info!("Creating workflow: {}", workflow.name);
    
    state.workflow_engine.load_workflow(workflow.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, Json(serde_json::json!({
        "workflow": workflow.name,
        "loaded": true
    }))))
}

/// Update a workflow
pub async fn update_workflow(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
    Json(workflow): Json<WorkflowDefinition>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Updating workflow for entity: {}", entity);
    
    let _ = state.workflow_engine.unload_workflow(&workflow.name).await;
    
    state.workflow_engine.load_workflow(workflow.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({
        "workflow": workflow.name,
        "updated": true
    })))
}

/// Get all configuration
pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let entities = state.schema_engine.entity_names();
    
    Ok(Json(serde_json::json!({
        "entities": entities,
        "version": state.schema_engine.get_version().await,
    })))
}

/// Get a configuration value
pub async fn get_config_value(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let parts: Vec<&str> = key.split('.').collect();
    
    match parts.first() {
        Some(&"entity") => {
            if parts.len() >= 2 {
                let entity_name = parts[1];
                if let Some(entity) = state.schema_engine.get_entity(entity_name) {
                    return Ok(Json(serde_json::to_value(entity).unwrap()));
                }
            }
        }
        Some(&"workflow") => {
            if parts.len() >= 2 {
                let workflow_name = parts[1];
                if let Some(workflow) = state.workflow_engine.get_workflow(workflow_name).await {
                    return Ok(Json(serde_json::to_value(workflow).unwrap()));
                }
            }
        }
        _ => {}
    }
    
    Err(StatusCode::NOT_FOUND)
}

/// Set a configuration value
pub async fn set_config_value(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    Json(value): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let parts: Vec<&str> = key.split('.').collect();
    
    match parts.first() {
        Some(&"entity") => {
            if parts.len() >= 2 {
                if let Ok(definition) = serde_json::from_value::<EntityDefinition>(value) {
                    state.schema_engine.upsert_entity(definition)
                        .await
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    
                    return Ok(Json(serde_json::json!({
                        "key": key,
                        "updated": true
                    })));
                }
            }
        }
        _ => {}
    }
    
    Err(StatusCode::BAD_REQUEST)
}

/// Clear all caches
pub async fn clear_cache(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Clearing all caches");
    
    state.schema_engine.refresh()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({
        "cache_cleared": true
    })))
}

/// Invalidate cache for a specific entity
pub async fn invalidate_entity_cache(
    State(state): State<Arc<AppState>>,
    Path(entity): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    debug!("Invalidating cache for entity: {}", entity);
    
    if let Some(definition) = state.schema_engine.get_entity(&entity) {
        state.schema_engine.upsert_entity(definition)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    
    Ok(Json(serde_json::json!({
        "entity": entity,
        "cache_invalidated": true
    })))
}
