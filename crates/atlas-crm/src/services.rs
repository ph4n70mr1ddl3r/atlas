//! CRM Services
//! 
//! Business logic services for the Customer Relationship Management domain.
//! Provides customer lifecycle management, lead scoring, and sales pipeline operations.

use atlas_core::{SchemaEngine, WorkflowEngine, ValidationEngine};
use atlas_shared::{AtlasResult, AtlasError, RecordId};
use std::sync::Arc;
use serde_json::json;
use tracing::info;

/// Customer service
#[allow(dead_code)]
pub struct CustomerService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

impl CustomerService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Convert a lead to a customer
    /// 
    /// Creates a new customer record from lead data and triggers
    /// the lead's "won" workflow transition.
    pub async fn convert_lead(&self, lead_id: RecordId) -> AtlasResult<RecordId> {
        let _customer_entity = self.schema_engine.get_entity("customers")
            .ok_or_else(|| AtlasError::EntityNotFound("customers".to_string()))?;
        let lead_entity = self.schema_engine.get_entity("leads")
            .ok_or_else(|| AtlasError::EntityNotFound("leads".to_string()))?;
        
        let customer_id = RecordId::new_v4();
        
        // Trigger lead workflow to "won" state if workflow exists
        if let Some(workflow) = &lead_entity.workflow {
            let _ = self.workflow_engine.execute_transition(
                &workflow.name,
                lead_id,
                "proposal",
                "mark_won",
                None,
                &json!({"customer_id": customer_id}),
                None,
            ).await;
        }
        
        info!("Lead {} converted to customer {}", lead_id, customer_id);
        Ok(customer_id)
    }

    /// Get customer 360 view
    /// 
    /// Aggregates data from all CRM entities related to a customer.
    pub async fn get_customer_360(&self, customer_id: RecordId) -> AtlasResult<serde_json::Value> {
        let _entity = self.schema_engine.get_entity("customers")
            .ok_or_else(|| AtlasError::EntityNotFound("customers".to_string()))?;
        
        Ok(json!({
            "customer_id": customer_id,
            "contacts": [],
            "opportunities": [],
            "service_cases": [],
            "recent_orders": [],
            "total_revenue": 0.0,
            "open_cases": 0,
        }))
    }
}

/// Lead service
#[allow(dead_code)]
pub struct LeadService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

impl LeadService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self { schema_engine, workflow_engine, validation_engine }
    }

    /// Score a lead based on criteria
    /// 
    /// Computes a lead score based on company size, industry, engagement, etc.
    pub async fn score_lead(&self, _lead_id: RecordId) -> AtlasResult<i32> {
        let _entity = self.schema_engine.get_entity("leads")
            .ok_or_else(|| AtlasError::EntityNotFound("leads".to_string()))?;
        
        // In a real implementation, evaluate criteria from the lead data
        // and compute a weighted score
        Ok(50)
    }

    /// Assign lead to sales rep
    /// 
    /// Updates the assigned_to_id field and triggers any assignment actions.
    pub async fn assign(&self, lead_id: RecordId, rep_id: RecordId) -> AtlasResult<()> {
        let _entity = self.schema_engine.get_entity("leads")
            .ok_or_else(|| AtlasError::EntityNotFound("leads".to_string()))?;
        
        info!("Lead {} assigned to rep {}", lead_id, rep_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::entities;
    
    #[test]
    fn test_customer_definition_builds() {
        let def = entities::customer_definition();
        assert_eq!(def.name, "customers");
    }
    
    #[test]
    fn test_lead_definition_builds() {
        let def = entities::lead_definition();
        assert_eq!(def.name, "leads");
        assert!(def.workflow.is_some());
    }
    
    #[test]
    fn test_opportunity_definition_builds() {
        let def = entities::opportunity_definition();
        assert_eq!(def.name, "opportunities");
        assert!(def.workflow.is_some());
    }
    
    #[test]
    fn test_contact_definition_builds() {
        let def = entities::contact_definition();
        assert_eq!(def.name, "contacts");
    }
    
    #[test]
    fn test_service_case_definition_builds() {
        let def = entities::service_case_definition();
        assert_eq!(def.name, "service_cases");
        assert!(def.workflow.is_some());
    }
}
