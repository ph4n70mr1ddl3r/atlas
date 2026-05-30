use std::sync::Arc;
use atlas_core::*;

#[derive(Clone)]
pub struct CoreState {
    pub schema_engine: Arc<SchemaEngine>,
    pub workflow_engine: Arc<WorkflowEngine>,
    pub validation_engine: Arc<ValidationEngine>,
    pub formula_engine: Arc<FormulaEngine>,
    pub security_engine: Arc<SecurityEngine>,
    pub audit_engine: Arc<AuditEngine>,
    pub notification_engine: Arc<NotificationEngine>,
    pub approval_engine: Arc<ApprovalEngine>,
    pub document_sequencing_engine: Arc<DocumentSequencingEngine>,
    pub transaction_calendar_engine: Arc<TransactionCalendarEngine>,
}
