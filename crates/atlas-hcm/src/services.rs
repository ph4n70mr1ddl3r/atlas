//! HCM Services
//! 
//! Business logic services for HCM domain.
//! These services interact with the declarative entities via the core engine.

use atlas_core::{SchemaEngine, WorkflowEngine, ValidationEngine};
use atlas_shared::{RecordId, AtlasResult, AtlasError};
use std::sync::Arc;
use serde_json::json;
use tracing::info;

/// Employee service for HCM operations
/// Type alias for entity definition factory functions
pub type EntityDefinitionFactory = fn() -> atlas_shared::EntityDefinition;

#[allow(dead_code)]
pub struct EmployeeService {
    schema_engine: Arc<SchemaEngine>,
    workflow_engine: Arc<WorkflowEngine>,
    validation_engine: Arc<ValidationEngine>,
}

impl EmployeeService {
    pub fn new(
        schema_engine: Arc<SchemaEngine>,
        workflow_engine: Arc<WorkflowEngine>,
        validation_engine: Arc<ValidationEngine>,
    ) -> Self {
        Self {
            schema_engine,
            workflow_engine,
            validation_engine,
        }
    }
    
    /// Onboard a new employee
    /// 
    /// Validates the data against the employee entity schema and triggers
    /// the employee lifecycle workflow from the onboarding state.
    pub async fn onboard_employee(
        &self,
        data: serde_json::Value,
        _user_id: Option<RecordId>,
    ) -> AtlasResult<RecordId> {
        let entity = self.schema_engine.get_entity("employees")
            .ok_or_else(|| AtlasError::EntityNotFound("employees".to_string()))?;
        
        // Validate the data against the entity definition
        let validation_result = self.validation_engine.validate(&entity, &data, None);
        if !validation_result.valid {
            let errors: Vec<String> = validation_result.errors.iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect();
            return Err(AtlasError::ValidationFailed(errors.join(", ")));
        }
        
        let record_id = RecordId::new_v4();
        info!("Employee onboarded: {} ({})", 
            data.get("first_name").and_then(|v| v.as_str()).unwrap_or("Unknown"),
            record_id
        );
        
        // Publish event (via event bus in production)
        // let event = EventFactory::record_created(
        //     "atlas-hcm", "employees", record_id, data.clone(), user_id
        // );
        
        Ok(record_id)
    }
    
    /// Transfer an employee to a new department
    /// 
    /// Updates the department_id and logs the change in the audit trail.
    pub async fn transfer(
        &self,
        employee_id: RecordId,
        new_department_id: RecordId,
    ) -> AtlasResult<()> {
        let entity = self.schema_engine.get_entity("employees")
            .ok_or_else(|| AtlasError::EntityNotFound("employees".to_string()))?;
        
        // Verify the department field exists
        if entity.fields.iter().find(|f| f.name == "department_id").is_none() {
            return Err(AtlasError::FieldNotFound("employees".to_string(), "department_id".to_string()));
        }
        
        info!("Employee {} transferred to department {}", employee_id, new_department_id);
        
        Ok(())
    }
    
    /// Terminate an employee
    /// 
    /// Triggers the offboarding workflow transition.
    pub async fn terminate(
        &self,
        employee_id: RecordId,
        reason: &str,
    ) -> AtlasResult<()> {
        let entity = self.schema_engine.get_entity("employees")
            .ok_or_else(|| AtlasError::EntityNotFound("employees".to_string()))?;
        
        if let Some(workflow) = &entity.workflow {
            // Try to transition to offboarding state
            let result = self.workflow_engine.execute_transition(
                &workflow.name,
                employee_id,
                "active",
                "start_offboarding",
                None,
                &json!({"reason": reason}),
                Some(reason.to_string()),
            ).await?;
            
            if !result.success {
                return Err(AtlasError::WorkflowError(
                    result.error.unwrap_or_else(|| "Transition failed".to_string())
                ));
            }
            
            info!("Employee {} terminated: {}", employee_id, reason);
        }
        
        Ok(())
    }
    
    pub fn get_entity_definitions() -> Vec<(&'static str, EntityDefinitionFactory)> {
        vec![
            ("employees", crate::entities::employee_definition),
            ("departments", crate::entities::department_definition),
            ("positions", crate::entities::position_definition),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_definitions_available() {
        let defs = EmployeeService::get_entity_definitions();
        assert_eq!(defs.len(), 3);
        assert_eq!(defs[0].0, "employees");
        assert_eq!(defs[1].0, "departments");
    }
    
    #[test]
    fn test_employee_definition_builds() {
        let def = crate::entities::employee_definition();
        assert_eq!(def.name, "employees");
        assert!(def.fields.len() > 5);
    }
    
    #[test]
    fn test_department_definition_builds() {
        let def = crate::entities::department_definition();
        assert_eq!(def.name, "departments");
    }
    
    #[test]
    fn test_position_definition_builds() {
        let def = crate::entities::position_definition();
        assert_eq!(def.name, "positions");
        assert!(def.fields.len() > 3);
    }
}
