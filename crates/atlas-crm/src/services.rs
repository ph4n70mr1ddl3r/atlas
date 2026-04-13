//! CRM Services

use atlas_core::{SchemaEngine, WorkflowEngine};
use std::sync::Arc;

/// Customer service
pub struct CustomerService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
}

impl CustomerService {
    pub fn new(schema_engine: Arc<SchemaEngine>, workflow_engine: Arc<WorkflowEngine>) -> Self {
        Self { schema_engine, workflow_engine }
    }
}

/// Lead service
pub struct LeadService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
}

impl LeadService {
    pub fn new(schema_engine: Arc<SchemaEngine>, workflow_engine: Arc<WorkflowEngine>) -> Self {
        Self { schema_engine, workflow_engine }
    }
}
