//! Workflow State Repository
//! 
//! Persistence for workflow states in PostgreSQL.

use atlas_shared::{AtlasError, AtlasResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use super::{WorkflowState, StateHistoryEntry};

/// Repository trait for workflow state persistence
#[async_trait]
pub trait WorkflowStateRepository: Send + Sync {
    /// Get the current workflow state for a record
    async fn get_state(&self, entity_type: &str, record_id: Uuid) -> AtlasResult<Option<WorkflowState>>;
    
    /// Save/update workflow state
    async fn save_state(&self, state: &WorkflowState) -> AtlasResult<()>;
    
    /// Get all records in a given state
    async fn get_records_in_state(&self, entity_type: &str, state_name: &str) -> AtlasResult<Vec<WorkflowState>>;
}

/// PostgreSQL implementation
pub struct PostgresWorkflowStateRepository {
    pool: PgPool,
}

impl PostgresWorkflowStateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkflowStateRepository for PostgresWorkflowStateRepository {
    async fn get_state(&self, entity_type: &str, record_id: Uuid) -> AtlasResult<Option<WorkflowState>> {
        let row = sqlx::query_as::<_, WorkflowStateRow>(
            r#"
            SELECT record_id, entity_type, workflow_name, current_state, state_type, history, metadata
            FROM _atlas.workflow_states
            WHERE entity_type = $1 AND record_id = $2
            "#
        )
        .bind(entity_type)
        .bind(record_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| r.into()))
    }
    
    async fn save_state(&self, state: &WorkflowState) -> AtlasResult<()> {
        let history_json = serde_json::to_value(&state.history)
            .map_err(|e| AtlasError::Internal(e.to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO _atlas.workflow_states (record_id, entity_type, workflow_name, current_state, state_type, history, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (entity_type, record_id) DO UPDATE SET
                workflow_name = EXCLUDED.workflow_name,
                current_state = EXCLUDED.current_state,
                state_type = EXCLUDED.state_type,
                history = EXCLUDED.history,
                metadata = EXCLUDED.metadata,
                updated_at = now()
            "#
        )
        .bind(state.record_id)
        .bind(&state.entity_type)
        .bind(&state.workflow_name)
        .bind(&state.current_state)
        .bind(format!("{:?}", state.state_type))
        .bind(&history_json)
        .bind(&state.metadata)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_records_in_state(&self, entity_type: &str, state_name: &str) -> AtlasResult<Vec<WorkflowState>> {
        let rows = sqlx::query_as::<_, WorkflowStateRow>(
            r#"
            SELECT record_id, entity_type, workflow_name, current_state, state_type, history, metadata
            FROM _atlas.workflow_states
            WHERE entity_type = $1 AND current_state = $2
            ORDER BY updated_at DESC
            "#
        )
        .bind(entity_type)
        .bind(state_name)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
}

#[derive(sqlx::FromRow)]
struct WorkflowStateRow {
    record_id: Uuid,
    entity_type: String,
    workflow_name: String,
    current_state: String,
    state_type: String,
    history: serde_json::Value,
    metadata: serde_json::Value,
}

impl From<WorkflowStateRow> for WorkflowState {
    fn from(row: WorkflowStateRow) -> Self {
        use atlas_shared::StateType;
        
        let state_type = match row.state_type.as_str() {
            "Initial" => StateType::Initial,
            "Working" => StateType::Working,
            "Final" => StateType::Final,
            _ => StateType::Working,
        };
        
        WorkflowState {
            record_id: row.record_id,
            entity_type: row.entity_type,
            workflow_name: row.workflow_name,
            current_state: row.current_state,
            state_type,
            history: serde_json::from_value(row.history).unwrap_or_default(),
            metadata: row.metadata,
        }
    }
}

/// In-memory workflow state repository for testing
pub struct InMemoryWorkflowStateRepository {
    states: std::sync::RwLock<std::collections::HashMap<(String, Uuid), WorkflowState>>,
}

impl InMemoryWorkflowStateRepository {
    pub fn new() -> Self {
        Self {
            states: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

impl Default for InMemoryWorkflowStateRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorkflowStateRepository for InMemoryWorkflowStateRepository {
    async fn get_state(&self, entity_type: &str, record_id: Uuid) -> AtlasResult<Option<WorkflowState>> {
        let states = self.states.read().unwrap();
        Ok(states.get(&(entity_type.to_string(), record_id)).cloned())
    }
    
    async fn save_state(&self, state: &WorkflowState) -> AtlasResult<()> {
        let mut states = self.states.write().unwrap();
        states.insert((state.entity_type.clone(), state.record_id), state.clone());
        Ok(())
    }
    
    async fn get_records_in_state(&self, entity_type: &str, state_name: &str) -> AtlasResult<Vec<WorkflowState>> {
        let states = self.states.read().unwrap();
        Ok(states.values()
            .filter(|s| s.entity_type == entity_type && s.current_state == state_name)
            .cloned()
            .collect())
    }
}
