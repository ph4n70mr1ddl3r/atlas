//! Workflow Engine Implementation

use atlas_shared::{WorkflowDefinition, RecordId, UserId, AtlasError, AtlasResult, StateType};
use super::{TransitionResult, AvailableTransitions, TransitionInfo};
use super::{GuardEvaluator, ActionExecutor};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Workflow engine for managing state machines
pub struct WorkflowEngine {
    workflows: Arc<RwLock<HashMap<String, WorkflowDefinition>>>,
    guard_evaluator: Arc<GuardEvaluator>,
    action_executor: Arc<ActionExecutor>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            guard_evaluator: Arc::new(GuardEvaluator::new()),
            action_executor: Arc::new(ActionExecutor::new()),
        }
    }
    
    /// Load a workflow definition
    pub async fn load_workflow(&self, workflow: WorkflowDefinition) -> AtlasResult<()> {
        let name = workflow.name.clone();
        info!("Loading workflow: {}", name);
        
        // Validate workflow
        self.validate_workflow(&workflow)?;
        
        let mut workflows = self.workflows.write().await;
        workflows.insert(name, workflow);
        
        Ok(())
    }
    
    /// Get a workflow definition
    pub async fn get_workflow(&self, name: &str) -> Option<WorkflowDefinition> {
        let workflows = self.workflows.read().await;
        workflows.get(name).cloned()
    }
    
    /// Get workflow for an entity by name
    pub async fn get_workflow_for_entity(&self, entity: &str) -> Option<WorkflowDefinition> {
        let workflows = self.workflows.read().await;
        // First try exact match on entity name, then fall back to first active workflow
        // that was loaded for this entity (by convention, workflow names often contain entity name)
        workflows.get(entity).cloned().or_else(|| {
            workflows.values().find(|w| w.is_active).cloned()
        })
    }
    
    /// Execute a workflow transition
    pub async fn execute_transition(
        &self,
        workflow_name: &str,
        record_id: RecordId,
        current_state: &str,
        action: &str,
        user: Option<&User>,
        record_data: &serde_json::Value,
        _comment: Option<String>,
    ) -> AtlasResult<TransitionResult> {
        let workflows = self.workflows.read().await;
        let workflow = workflows.get(workflow_name)
            .ok_or_else(|| AtlasError::WorkflowError(format!("Workflow not found: {}", workflow_name)))?;
        
        // Find the transition
        let transition = workflow.transitions.iter()
            .find(|t| t.from_state == current_state && t.action == action)
            .ok_or_else(|| AtlasError::InvalidStateTransition(
                current_state.to_string(), 
                action.to_string()
            ))?;
        
        // Check user roles if required
        if !transition.required_roles.is_empty() {
            if let Some(u) = user {
                let has_role = u.roles.iter().any(|r| transition.required_roles.contains(r));
                if !has_role {
                    return Ok(TransitionResult::failure(
                        current_state,
                        &transition.to_state,
                        action,
                        format!("User lacks required roles: {:?}", transition.required_roles)
                    ));
                }
            } else {
                return Ok(TransitionResult::failure(
                    current_state,
                    &transition.to_state,
                    action,
                    "Authentication required".to_string()
                ));
            }
        }
        
        // Evaluate guards
        for guard in &transition.guards {
            let result = self.guard_evaluator.evaluate(guard, record_data);
            if !result.passed {
                return Ok(TransitionResult::failure(
                    current_state,
                    &transition.to_state,
                    action,
                    result.message.unwrap_or_else(|| "Guard condition not met".to_string())
                ));
            }
        }
        
        // Get target state
        let to_state = &transition.to_state;
        
        // Execute exit actions for current state
        let mut executed_actions = vec![];
        if let Some(from_state) = workflow.states.iter().find(|s| s.name == current_state) {
            for action_def in &from_state.exit_actions {
                match self.action_executor.execute(action_def, record_id, record_data).await {
                    Ok(result) => executed_actions.push(result.action_name),
                    Err(e) => {
                        warn!("Action failed: {:?}", e);
                    }
                }
            }
        }
        
        // Execute transition actions
        for action_def in &transition.entry_actions {
            match self.action_executor.execute(action_def, record_id, record_data).await {
                Ok(result) => executed_actions.push(result.action_name),
                Err(e) => {
                    warn!("Action failed: {:?}", e);
                }
            }
        }
        
        // Execute entry actions for target state
        if let Some(to_state_def) = workflow.states.iter().find(|s| s.name.as_str() == to_state) {
            for action_def in &to_state_def.entry_actions {
                match self.action_executor.execute(action_def, record_id, record_data).await {
                    Ok(result) => executed_actions.push(result.action_name),
                    Err(e) => {
                        warn!("Action failed: {:?}", e);
                    }
                }
            }
        }
        
        info!(
            "Transition executed: {} -> {} (action: {})",
            current_state, to_state, action
        );
        
        Ok(TransitionResult::success(current_state, to_state, action, executed_actions))
    }
    
    /// Get available transitions for a state
    pub async fn get_available_transitions(
        &self,
        workflow_name: &str,
        current_state: &str,
        user: Option<&User>,
    ) -> AtlasResult<AvailableTransitions> {
        let workflows = self.workflows.read().await;
        let workflow = workflows.get(workflow_name)
            .ok_or_else(|| AtlasError::WorkflowError(format!("Workflow not found: {}", workflow_name)))?;
        
        let mut transitions = vec![];
        
        for t in &workflow.transitions {
            if t.from_state != current_state {
                continue;
            }
            
            // Check role requirements
            let mut has_access = t.required_roles.is_empty();
            if let Some(u) = user {
                has_access = u.roles.iter().any(|r| t.required_roles.contains(r));
            }
            
            if has_access {
                transitions.push(TransitionInfo {
                    name: t.name.clone(),
                    action: t.action.clone(),
                    action_label: t.action_label.clone(),
                    to_state: t.to_state.clone(),
                    required_roles: t.required_roles.clone(),
                    has_guards: !t.guards.is_empty(),
                });
            }
        }
        
        Ok(AvailableTransitions {
            current_state: current_state.to_string(),
            transitions,
        })
    }
    
    /// Validate a workflow definition
    fn validate_workflow(&self, workflow: &WorkflowDefinition) -> AtlasResult<()> {
        // Initial state must exist
        if !workflow.states.iter().any(|s| s.name == workflow.initial_state) {
            return Err(AtlasError::WorkflowError(
                "Initial state not found".to_string()
            ));
        }
        
        // Initial state must be of type Initial
        if let Some(initial) = workflow.states.iter().find(|s| s.name == workflow.initial_state) {
            if initial.state_type != StateType::Initial {
                return Err(AtlasError::WorkflowError(
                    "Initial state must have state_type = Initial".to_string()
                ));
            }
        }
        
        // All transitions must reference valid states
        for t in &workflow.transitions {
            if !workflow.states.iter().any(|s| s.name == t.from_state) {
                return Err(AtlasError::WorkflowError(
                    format!("Transition references unknown from_state: {}", t.from_state)
                ));
            }
            if !workflow.states.iter().any(|s| s.name == t.to_state) {
                return Err(AtlasError::WorkflowError(
                    format!("Transition references unknown to_state: {}", t.to_state)
                ));
            }
        }
        
        // There must be at least one final state
        let final_states = workflow.states.iter()
            .filter(|s| s.state_type == StateType::Final)
            .count();
        if final_states == 0 {
            return Err(AtlasError::WorkflowError(
                "Workflow must have at least one final state".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Check if a state is terminal (no outgoing transitions)
    pub async fn is_terminal_state(&self, workflow_name: &str, state: &str) -> bool {
        if let Some(workflow) = self.get_workflow(workflow_name).await {
            !workflow.transitions.iter().any(|t| t.from_state == state)
        } else {
            false
        }
    }
    
    /// Unload a workflow
    pub async fn unload_workflow(&self, name: &str) -> AtlasResult<()> {
        let mut workflows = self.workflows.write().await;
        workflows.remove(name);
        Ok(())
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// User context for authorization
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub roles: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_shared::{WorkflowDefinition, StateDefinition, TransitionDefinition, StateType};
    
    fn create_test_workflow() -> WorkflowDefinition {
        WorkflowDefinition {
            id: Some(uuid::Uuid::new_v4()),
            name: "test_workflow".to_string(),
            initial_state: "draft".to_string(),
            states: vec![
                StateDefinition {
                    name: "draft".to_string(),
                    label: "Draft".to_string(),
                    state_type: StateType::Initial,
                    entry_actions: vec![],
                    exit_actions: vec![],
                    metadata: serde_json::Value::Null,
                },
                StateDefinition {
                    name: "review".to_string(),
                    label: "Under Review".to_string(),
                    state_type: StateType::Working,
                    entry_actions: vec![],
                    exit_actions: vec![],
                    metadata: serde_json::Value::Null,
                },
                StateDefinition {
                    name: "approved".to_string(),
                    label: "Approved".to_string(),
                    state_type: StateType::Final,
                    entry_actions: vec![],
                    exit_actions: vec![],
                    metadata: serde_json::Value::Null,
                },
                StateDefinition {
                    name: "rejected".to_string(),
                    label: "Rejected".to_string(),
                    state_type: StateType::Final,
                    entry_actions: vec![],
                    exit_actions: vec![],
                    metadata: serde_json::Value::Null,
                },
            ],
            transitions: vec![
                TransitionDefinition {
                    name: "submit".to_string(),
                    from_state: "draft".to_string(),
                    to_state: "review".to_string(),
                    action: "submit".to_string(),
                    action_label: Some("Submit for Review".to_string()),
                    guards: vec![],
                    required_roles: vec![],
                    entry_actions: vec![],
                    metadata: serde_json::Value::Null,
                },
                TransitionDefinition {
                    name: "approve".to_string(),
                    from_state: "review".to_string(),
                    to_state: "approved".to_string(),
                    action: "approve".to_string(),
                    action_label: Some("Approve".to_string()),
                    guards: vec![],
                    required_roles: vec!["approver".to_string()],
                    entry_actions: vec![],
                    metadata: serde_json::Value::Null,
                },
                TransitionDefinition {
                    name: "reject".to_string(),
                    from_state: "review".to_string(),
                    to_state: "rejected".to_string(),
                    action: "reject".to_string(),
                    action_label: Some("Reject".to_string()),
                    guards: vec![],
                    required_roles: vec!["approver".to_string()],
                    entry_actions: vec![],
                    metadata: serde_json::Value::Null,
                },
            ],
            is_active: true,
        }
    }
    
    #[tokio::test]
    async fn test_load_workflow() {
        let engine = WorkflowEngine::new();
        let workflow = create_test_workflow();
        
        engine.load_workflow(workflow.clone()).await.unwrap();
        
        let loaded = engine.get_workflow("test_workflow").await;
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, "test_workflow");
    }
    
    #[tokio::test]
    async fn test_execute_transition() {
        let engine = WorkflowEngine::new();
        let workflow = create_test_workflow();
        engine.load_workflow(workflow).await.unwrap();
        
        let result = engine.execute_transition(
            "test_workflow",
            uuid::Uuid::new_v4(),
            "draft",
            "submit",
            None,
            &serde_json::json!({}),
            None,
        ).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.from_state, "draft");
        assert_eq!(result.to_state, "review");
        assert_eq!(result.action, "submit");
    }
    
    #[tokio::test]
    async fn test_transition_with_role_check() {
        let engine = WorkflowEngine::new();
        let workflow = create_test_workflow();
        engine.load_workflow(workflow).await.unwrap();
        
        // User without approver role
        let user = User {
            id: uuid::Uuid::new_v4(),
            roles: vec!["viewer".to_string()],
        };
        
        let result = engine.execute_transition(
            "test_workflow",
            uuid::Uuid::new_v4(),
            "review",
            "approve",
            Some(&user),
            &serde_json::json!({}),
            None,
        ).await.unwrap();
        
        assert!(!result.success);
        assert!(result.error.unwrap().contains("required roles"));
    }
    
    #[tokio::test]
    async fn test_available_transitions() {
        let engine = WorkflowEngine::new();
        let workflow = create_test_workflow();
        engine.load_workflow(workflow).await.unwrap();
        
        let transitions = engine.get_available_transitions(
            "test_workflow",
            "draft",
            None,
        ).await.unwrap();
        
        assert_eq!(transitions.current_state, "draft");
        assert_eq!(transitions.transitions.len(), 1);
        assert_eq!(transitions.transitions[0].action, "submit");
    }
    
    #[tokio::test]
    async fn test_invalid_transition() {
        let engine = WorkflowEngine::new();
        let workflow = create_test_workflow();
        engine.load_workflow(workflow).await.unwrap();
        
        let result = engine.execute_transition(
            "test_workflow",
            uuid::Uuid::new_v4(),
            "draft",
            "approve",  // Can't approve from draft
            None,
            &serde_json::json!({}),
            None,
        ).await;
        
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_terminal_state() {
        let engine = WorkflowEngine::new();
        let workflow = create_test_workflow();
        engine.load_workflow(workflow).await.unwrap();
        
        assert!(!engine.is_terminal_state("test_workflow", "draft").await);
        assert!(!engine.is_terminal_state("test_workflow", "review").await);
        assert!(engine.is_terminal_state("test_workflow", "approved").await);
        assert!(engine.is_terminal_state("test_workflow", "rejected").await);
    }
}
