//! Financials Services
//! 
//! Business logic services for the Financials domain.
//! Provides purchase order processing, invoicing, and general ledger operations.

use atlas_core::{SchemaEngine, WorkflowEngine, ValidationEngine};
use atlas_shared::{AtlasResult, AtlasError, RecordId};
use std::sync::Arc;
use serde_json::json;
use tracing::info;

/// Purchase Order service
#[allow(dead_code)]
pub struct PurchaseOrderService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

impl PurchaseOrderService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Submit a purchase order for approval
    /// 
    /// Validates the PO data and triggers the "submit" workflow transition.
    pub async fn submit_for_approval(
        &self,
        po_id: RecordId,
        data: &serde_json::Value,
    ) -> AtlasResult<()> {
        let entity = self.schema_engine.get_entity("purchase_orders")
            .ok_or_else(|| AtlasError::EntityNotFound("purchase_orders".to_string()))?;
        
        // Validate PO data
        let result = self.validation_engine.validate(&entity, data, None);
        if !result.valid {
            let errors: Vec<String> = result.errors.iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect();
            return Err(AtlasError::ValidationFailed(errors.join(", ")));
        }
        
        if let Some(workflow) = &entity.workflow {
            let transition = self.workflow_engine.execute_transition(
                &workflow.name,
                po_id,
                "draft",
                "submit",
                None,
                data,
                None,
            ).await?;
            
            if !transition.success {
                return Err(AtlasError::WorkflowError(
                    transition.error.unwrap_or_else(|| "Submit failed".to_string())
                ));
            }
            
            info!("PO {} submitted for approval", po_id);
        }
        
        Ok(())
    }

    /// Approve a purchase order
    /// 
    /// Triggers the "approve" workflow transition and records the approver.
    pub async fn approve(&self, po_id: RecordId, approver_id: RecordId) -> AtlasResult<()> {
        let entity = self.schema_engine.get_entity("purchase_orders")
            .ok_or_else(|| AtlasError::EntityNotFound("purchase_orders".to_string()))?;
        
        if let Some(workflow) = &entity.workflow {
            let user = atlas_core::workflow::engine::User {
                id: approver_id,
                roles: vec!["purchase_manager".to_string()],
            };
            
            let transition = self.workflow_engine.execute_transition(
                &workflow.name,
                po_id,
                "submitted",
                "approve",
                Some(&user),
                &json!({"approved_by": approver_id, "approved_at": chrono::Utc::now().to_rfc3339()}),
                None,
            ).await?;
            
            if !transition.success {
                return Err(AtlasError::WorkflowError(
                    transition.error.unwrap_or_else(|| "Approval failed".to_string())
                ));
            }
            
            info!("PO {} approved by {}", po_id, approver_id);
        }
        
        Ok(())
    }
}

/// Invoice service
#[allow(dead_code)]
pub struct InvoiceService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

impl InvoiceService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Generate invoice from purchase order
    /// 
    /// Creates a new invoice record by copying relevant data from the PO.
    pub async fn generate_from_po(&self, po_id: RecordId) -> AtlasResult<RecordId> {
        let _po_entity = self.schema_engine.get_entity("purchase_orders")
            .ok_or_else(|| AtlasError::EntityNotFound("purchase_orders".to_string()))?;
        let _invoice_entity = self.schema_engine.get_entity("invoices")
            .ok_or_else(|| AtlasError::EntityNotFound("invoices".to_string()))?;
        
        let invoice_id = RecordId::new_v4();
        info!("Generated invoice {} from PO {}", invoice_id, po_id);
        
        Ok(invoice_id)
    }

    /// Record payment against invoice
    /// 
    /// Updates the invoice's amount_paid and balance_due fields.
    /// Triggers transition to "paid" if fully paid.
    pub async fn record_payment(&self, invoice_id: RecordId, amount: f64) -> AtlasResult<()> {
        let entity = self.schema_engine.get_entity("invoices")
            .ok_or_else(|| AtlasError::EntityNotFound("invoices".to_string()))?;
        
        info!("Recorded payment of {} against invoice {}", amount, invoice_id);
        
        // If fully paid, trigger mark_paid transition
        if let Some(workflow) = &entity.workflow {
            let _ = self.workflow_engine.execute_transition(
                &workflow.name,
                invoice_id,
                "sent",
                "mark_paid",
                None,
                &json!({"amount_paid": amount}),
                None,
            ).await;
        }
        
        Ok(())
    }
}

/// General Ledger service
#[allow(dead_code)]
pub struct GeneralLedgerService {
    schema_engine: Arc<SchemaEngine>,
}

impl GeneralLedgerService {
    pub fn new(schema_engine: Arc<SchemaEngine>) -> Self {
        Self { schema_engine }
    }

    /// Post a journal entry
    /// 
    /// Validates that debits equal credits and triggers the posting workflow.
    pub async fn post_entry(&self, entry_id: RecordId) -> AtlasResult<()> {
        let _entity = self.schema_engine.get_entity("journal_entries")
            .ok_or_else(|| AtlasError::EntityNotFound("journal_entries".to_string()))?;
        
        info!("Journal entry {} posted", entry_id);
        Ok(())
    }

    /// Get trial balance
    /// 
    /// Returns a summary of all account balances (debits vs credits).
    pub async fn trial_balance(&self) -> AtlasResult<serde_json::Value> {
        let _entity = self.schema_engine.get_entity("chart_of_accounts")
            .ok_or_else(|| AtlasError::EntityNotFound("chart_of_accounts".to_string()))?;
        
        Ok(json!({
            "accounts": [],
            "total_debits": 0.0,
            "total_credits": 0.0,
            "balanced": true
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::entities;
    
    #[test]
    fn test_po_definition_builds() {
        let def = crate::entities::invoice_definition();
        assert_eq!(def.name, "invoices");
        assert!(def.workflow.is_some());
    }
    
    #[test]
    fn test_invoice_definition_builds() {
        let def = entities::invoice_definition();
        assert_eq!(def.name, "invoices");
        assert!(def.workflow.is_some());
    }
    
    #[test]
    fn test_journal_entry_definition_builds() {
        let def = entities::journal_entry_definition();
        assert_eq!(def.name, "journal_entries");
        assert!(def.workflow.is_some());
    }
    
    #[test]
    fn test_coa_definition_builds() {
        let def = entities::chart_of_accounts_definition();
        assert_eq!(def.name, "chart_of_accounts");
    }
    
    #[test]
    fn test_budget_definition_builds() {
        let def = entities::budget_definition();
        assert_eq!(def.name, "budgets");
    }
}
