//! Projects Services

use atlas_core::{SchemaEngine, WorkflowEngine, ValidationEngine};
use atlas_shared::{AtlasResult, RecordId};
use std::sync::Arc;

/// Project service
pub struct ProjectService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

impl ProjectService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Get project dashboard data
    pub async fn dashboard(&self, _project_id: RecordId) -> AtlasResult<serde_json::Value> {
        Ok(serde_json::json!({
            "tasks_by_status": {},
            "budget_summary": {},
            "timeline": {}
        }))
    }

    /// Calculate project progress
    pub async fn calculate_progress(&self, _project_id: RecordId) -> AtlasResult<f64> {
        Ok(0.0)
    }
}

/// Task service
pub struct TaskService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
}

impl TaskService {
    pub fn new(schema_engine: Arc<SchemaEngine>, workflow_engine: Arc<WorkflowEngine>) -> Self {
        Self { schema_engine, workflow_engine }
    }

    /// Get tasks by assignee
    pub async fn get_tasks_by_assignee(&self, _assignee_id: RecordId) -> AtlasResult<Vec<serde_json::Value>> {
        Ok(vec![])
    }

    /// Get overdue tasks
    pub async fn get_overdue_tasks(&self) -> AtlasResult<Vec<serde_json::Value>> {
        Ok(vec![])
    }
}
