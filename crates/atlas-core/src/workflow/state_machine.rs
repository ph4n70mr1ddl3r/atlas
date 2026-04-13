//! State Machine
//! 
//! Represents the runtime state of a workflow instance.

use atlas_shared::{RecordId, StateType};
use super::{WorkflowState, StateHistoryEntry};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Runtime state machine for a workflow instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachine {
    pub record_id: RecordId,
    pub entity_type: String,
    pub workflow_name: String,
    pub current_state: String,
    pub current_state_type: StateType,
    pub history: Vec<StateHistoryEntry>,
    pub metadata: serde_json::Value,
}

impl StateMachine {
    /// Create a new state machine with the initial state
    pub fn new(record_id: RecordId, entity_type: &str, workflow_name: &str, initial_state: &str) -> Self {
        Self {
            record_id,
            entity_type: entity_type.to_string(),
            workflow_name: workflow_name.to_string(),
            current_state: initial_state.to_string(),
            current_state_type: StateType::Initial,
            history: vec![],
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    /// Create from stored workflow state
    pub fn from_state(state: WorkflowState) -> Self {
        Self {
            record_id: state.record_id,
            entity_type: state.entity_type,
            workflow_name: state.workflow_name,
            current_state: state.current_state,
            current_state_type: state.state_type,
            history: state.history,
            metadata: state.metadata,
        }
    }
    
    /// Convert back to stored workflow state
    pub fn into_state(self) -> WorkflowState {
        WorkflowState {
            record_id: self.record_id,
            entity_type: self.entity_type,
            workflow_name: self.workflow_name,
            current_state: self.current_state,
            state_type: self.current_state_type,
            history: self.history,
            metadata: self.metadata,
        }
    }
    
    /// Transition to a new state
    pub fn transition(
        &mut self, 
        to_state: &str, 
        state_type: StateType,
        action: &str, 
        user_id: Option<uuid::Uuid>,
        comment: Option<String>,
    ) {
        let entry = StateHistoryEntry {
            from_state: Some(self.current_state.clone()),
            to_state: to_state.to_string(),
            action: action.to_string(),
            performed_by: user_id,
            comment,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
        };
        
        self.history.push(entry);
        self.current_state = to_state.to_string();
        self.current_state_type = state_type;
    }
    
    /// Check if the state machine is in a terminal state
    pub fn is_terminal(&self) -> bool {
        self.current_state_type == StateType::Final
    }
    
    /// Get the transition history as a formatted string
    pub fn history_summary(&self) -> String {
        self.history.iter()
            .map(|h| {
                let from = h.from_state.as_deref().unwrap_or("(start)");
                format!("{}: {} -> {} ({})", 
                    h.timestamp.format("%Y-%m-%d %H:%M"),
                    from,
                    h.to_state,
                    h.action
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Get the last transition
    pub fn last_transition(&self) -> Option<&StateHistoryEntry> {
        self.history.last()
    }
    
    /// Get transition count
    pub fn transition_count(&self) -> usize {
        self.history.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_state_machine_creation() {
        let record_id = uuid::Uuid::new_v4();
        let sm = StateMachine::new(record_id, "test_entity", "test_workflow", "draft");
        
        assert_eq!(sm.record_id, record_id);
        assert_eq!(sm.current_state, "draft");
        assert_eq!(sm.current_state_type, StateType::Initial);
        assert!(sm.history.is_empty());
        assert!(!sm.is_terminal());
    }
    
    #[test]
    fn test_state_machine_transition() {
        let record_id = uuid::Uuid::new_v4();
        let mut sm = StateMachine::new(record_id, "test_entity", "test_workflow", "draft");
        
        sm.transition(
            "review", 
            StateType::Working, 
            "submit", 
            Some(uuid::Uuid::new_v4()),
            None
        );
        
        assert_eq!(sm.current_state, "review");
        assert_eq!(sm.current_state_type, StateType::Working);
        assert_eq!(sm.history.len(), 1);
        assert_eq!(sm.history[0].action, "submit");
    }
    
    #[test]
    fn test_state_machine_terminal() {
        let record_id = uuid::Uuid::new_v4();
        let mut sm = StateMachine::new(record_id, "test_entity", "test_workflow", "draft");
        
        assert!(!sm.is_terminal());
        
        sm.transition("approved", StateType::Final, "approve", None, None);
        
        assert!(sm.is_terminal());
    }
    
    #[test]
    fn test_history_summary() {
        let record_id = uuid::Uuid::new_v4();
        let mut sm = StateMachine::new(record_id, "test_entity", "test_workflow", "draft");
        
        sm.transition("review", StateType::Working, "submit", None, None);
        sm.transition("approved", StateType::Final, "approve", None, None);
        
        let summary = sm.history_summary();
        assert!(summary.contains("draft -> review"));
        assert!(summary.contains("review -> approved"));
    }
    
    #[test]
    fn test_state_conversion() {
        let record_id = uuid::Uuid::new_v4();
        let sm = StateMachine::new(record_id, "test_entity", "test_workflow", "draft");
        let state: WorkflowState = sm.clone().into_state();
        let sm2 = StateMachine::from_state(state);
        
        assert_eq!(sm.record_id, sm2.record_id);
        assert_eq!(sm.current_state, sm2.current_state);
    }
}
