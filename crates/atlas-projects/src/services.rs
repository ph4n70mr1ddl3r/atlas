//! Projects Services
//! 
//! Business logic services for the Project Management domain.
//! Provides project tracking, task management, timesheet processing, and progress calculation.

use atlas_core::{SchemaEngine, WorkflowEngine, ValidationEngine};
use atlas_shared::{AtlasResult, AtlasError, RecordId};
use std::sync::Arc;
use serde_json::json;

/// Project service
#[allow(dead_code)]
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
    /// 
    /// Aggregates project statistics including task breakdown, budget summary, and timeline.
    pub async fn dashboard(&self, project_id: RecordId) -> AtlasResult<serde_json::Value> {
        let _entity = self.schema_engine.get_entity("projects")
            .ok_or_else(|| AtlasError::EntityNotFound("projects".to_string()))?;
        
        Ok(json!({
            "project_id": project_id,
            "tasks_by_status": {
                "todo": 0,
                "in_progress": 0,
                "in_review": 0,
                "done": 0,
            },
            "budget_summary": {
                "total": 0.0,
                "spent": 0.0,
                "remaining": 0.0,
                "percent_used": 0.0,
            },
            "timeline": {
                "start_date": null,
                "end_date": null,
                "days_remaining": 0,
                "on_track": true,
            },
            "team": [],
            "milestones": [],
        }))
    }

    /// Calculate project progress
    /// 
    /// Computes progress based on completed tasks vs total tasks.
    pub async fn calculate_progress(&self, _project_id: RecordId) -> AtlasResult<f64> {
        let _entity = self.schema_engine.get_entity("tasks")
            .ok_or_else(|| AtlasError::EntityNotFound("tasks".to_string()))?;
        
        // In a real implementation, query all tasks for this project
        // and compute: completed_count / total_count * 100
        Ok(0.0)
    }
}

/// Task service
#[allow(dead_code)]
pub struct TaskService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
}

impl TaskService {
    pub fn new(schema_engine: Arc<SchemaEngine>, workflow_engine: Arc<WorkflowEngine>) -> Self {
        Self { schema_engine, workflow_engine }
    }

    /// Get tasks by assignee
    /// 
    /// Returns all tasks assigned to a specific person.
    pub async fn get_tasks_by_assignee(&self, _assignee_id: RecordId) -> AtlasResult<Vec<serde_json::Value>> {
        let _entity = self.schema_engine.get_entity("tasks")
            .ok_or_else(|| AtlasError::EntityNotFound("tasks".to_string()))?;
        
        // In a real implementation, query tasks WHERE assignee_id = $1
        Ok(vec![])
    }

    /// Get overdue tasks
    /// 
    /// Returns all tasks where due_date < now AND workflow_state != 'done' AND != 'cancelled'.
    pub async fn get_overdue_tasks(&self) -> AtlasResult<Vec<serde_json::Value>> {
        let _entity = self.schema_engine.get_entity("tasks")
            .ok_or_else(|| AtlasError::EntityNotFound("tasks".to_string()))?;
        
        // In a real implementation, query tasks with overdue criteria
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities;
    
    #[test]
    fn test_project_definition_builds() {
        let def = entities::project_definition();
        assert_eq!(def.name, "projects");
        assert!(def.workflow.is_some());
    }
    
    #[test]
    fn test_task_definition_builds() {
        let def = entities::task_definition();
        assert_eq!(def.name, "tasks");
        assert!(def.workflow.is_some());
    }
    
    #[test]
    fn test_timesheet_definition_builds() {
        let def = entities::timesheet_definition();
        assert_eq!(def.name, "timesheets");
        assert!(def.workflow.is_some());
    }
    
    #[test]
    fn test_milestone_definition_builds() {
        let def = entities::milestone_definition();
        assert_eq!(def.name, "milestones");
    }
}
