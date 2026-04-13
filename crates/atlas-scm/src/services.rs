//! SCM Services

use atlas_core::{SchemaEngine, WorkflowEngine};
use std::sync::Arc;

/// Inventory service
pub struct InventoryService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
}

impl InventoryService {
    pub fn new(schema_engine: Arc<SchemaEngine>, workflow_engine: Arc<WorkflowEngine>) -> Self {
        Self { schema_engine, workflow_engine }
    }
}

/// Supplier service
pub struct SupplierService {
    schema_engine: Arc<SchemaEngine>,
}

impl SupplierService {
    pub fn new(schema_engine: Arc<SchemaEngine>) -> Self {
        Self { schema_engine }
    }
}
