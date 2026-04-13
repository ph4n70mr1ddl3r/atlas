//! Security Engine
//! 
//! Row-level and field-level security enforcement.

mod engine;
mod rls;

pub use engine::SecurityEngine;
pub use rls::*;

use atlas_shared::{SecurityPolicy, SecurityRule};
use serde::{Deserialize, Serialize};

/// Security context for access control
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub user_id: Option<uuid::Uuid>,
    pub organization_id: Option<uuid::Uuid>,
    pub roles: Vec<String>,
    pub session_id: Option<uuid::Uuid>,
}

/// Access control decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecision {
    pub allowed: bool,
    pub reason: Option<String>,
}

/// Field-level security
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldSecurity {
    pub read_roles: Vec<String>,
    pub write_roles: Vec<String>,
    pub hidden: bool,
}

/// Check result for a specific field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldCheck {
    pub field: String,
    pub can_read: bool,
    pub can_write: bool,
    pub visible: bool,
}
