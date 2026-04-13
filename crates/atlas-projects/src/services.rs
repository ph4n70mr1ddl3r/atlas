//! Projects Services

use atlas_core::{SchemaEngine, WorkflowEngine};
use std::sync::Arc;

/// Project service
pub struct ProjectService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
}

impl ProjectService {
    pub fn new(schema_engine: Arc<SchemaEngine>, workflow_engine: Arc<WorkflowEngine>) -> Self {
        Self { schema_engine, workflow_engine }
    }
}

/// Task service
pub struct TaskService {
    schema_engine: Arc<SchemaEngine>,
}

impl TaskService {
    pub fn new(schema_engine: Arc<SchemaEngine>) -> Self {
        Self { schema_engine }
    }
}
