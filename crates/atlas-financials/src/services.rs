//! Financials Services

use atlas_core::{SchemaEngine, WorkflowEngine};
use std::sync::Arc;

/// Purchase Order service
pub struct PurchaseOrderService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
}

impl PurchaseOrderService {
    pub fn new(schema_engine: Arc<SchemaEngine>, workflow_engine: Arc<WorkflowEngine>) -> Self {
        Self { schema_engine, workflow_engine }
    }
}

/// Invoice service
pub struct InvoiceService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
}

impl InvoiceService {
    pub fn new(schema_engine: Arc<SchemaEngine>, workflow_engine: Arc<WorkflowEngine>) -> Self {
        Self { schema_engine, workflow_engine }
    }
}
