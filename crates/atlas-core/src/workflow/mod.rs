//! Workflow Engine
//! 
//! State machine execution for declarative workflows.
//! Handles transitions, guards, and actions.

mod engine;
mod state_machine;
mod guards;
mod actions;
mod repository;

pub use engine::WorkflowEngine;
pub use state_machine::StateMachine;
pub use guards::{GuardEvaluator, GuardResult};
pub use actions::{ActionExecutor, ActionResult};
pub use repository::{WorkflowStateRepository, PostgresWorkflowStateRepository, InMemoryWorkflowStateRepository};

use atlas_shared::StateType;
use atlas_shared::{RecordId, UserId};
use serde::{Deserialize, Serialize};

/// Represents the current state of a workflow instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowState {
    pub record_id: RecordId,
    pub entity_type: String,
    pub workflow_name: String,
    pub current_state: String,
    pub state_type: StateType,
    pub history: Vec<StateHistoryEntry>,
    pub metadata: serde_json::Value,
}

/// Entry in the workflow history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateHistoryEntry {
    pub from_state: Option<String>,
    pub to_state: String,
    pub action: String,
    pub performed_by: Option<UserId>,
    pub comment: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

/// Result of executing a transition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionResult {
    pub success: bool,
    pub from_state: String,
    pub to_state: String,
    pub action: String,
    pub message: Option<String>,
    pub executed_actions: Vec<String>,
    pub error: Option<String>,
}

impl TransitionResult {
    pub fn success(from: &str, to: &str, action: &str, actions: Vec<String>) -> Self {
        Self {
            success: true,
            from_state: from.to_string(),
            to_state: to.to_string(),
            action: action.to_string(),
            message: None,
            executed_actions: actions,
            error: None,
        }
    }
    
    pub fn failure(from: &str, to: &str, action: &str, error: String) -> Self {
        Self {
            success: false,
            from_state: from.to_string(),
            to_state: to.to_string(),
            action: action.to_string(),
            message: None,
            executed_actions: vec![],
            error: Some(error),
        }
    }
}

/// Available transitions from a state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableTransitions {
    pub current_state: String,
    pub transitions: Vec<TransitionInfo>,
}

/// Information about a single transition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransitionInfo {
    pub name: String,
    pub action: String,
    pub action_label: Option<String>,
    pub to_state: String,
    pub required_roles: Vec<String>,
    pub has_guards: bool,
}
