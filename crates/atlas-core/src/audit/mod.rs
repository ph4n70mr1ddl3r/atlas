//! Audit Engine
//! 
//! Change tracking and audit trail for all data modifications.

mod engine;
mod repository;

pub use engine::AuditEngine;
pub use repository::{AuditRepository, PostgresAuditRepository};

use atlas_shared::{AuditEntry, AuditAction, RecordId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Audit query filters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditQuery {
    pub entity_type: Option<String>,
    pub entity_id: Option<RecordId>,
    pub action: Option<AuditAction>,
    pub user_id: Option<UserId>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Audit summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditSummary {
    pub total_actions: i64,
    pub actions_by_type: std::collections::HashMap<String, i64>,
    pub actions_by_user: std::collections::HashMap<String, i64>,
    pub recent_activity: Vec<AuditEntry>,
}

/// Changes between two versions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSet {
    pub entity_id: RecordId,
    pub changes: Vec<FieldChange>,
}

/// A single field change
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FieldChange {
    pub field: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
}

impl FieldChange {
    pub fn new(field: &str, old_val: Option<serde_json::Value>, new_val: Option<serde_json::Value>) -> Self {
        Self {
            field: field.to_string(),
            old_value: old_val,
            new_value: new_val,
        }
    }
    
    /// Compute changes between two JSON values
    pub fn compute_changes(old_data: &serde_json::Value, new_data: &serde_json::Value) -> Vec<FieldChange> {
        let mut changes = vec![];
        
        let empty_old = serde_json::Map::new();
        let empty_new = serde_json::Map::new();
        let old_obj = old_data.as_object().unwrap_or(&empty_old);
        let new_obj = new_data.as_object().unwrap_or(&empty_new);
        
        // Check all keys in new data
        for (key, new_val) in new_obj {
            let old_val = old_obj.get(key);
            
            if !values_equal(old_val, new_val) {
                changes.push(FieldChange::new(
                    key,
                    old_val.cloned(),
                    Some(new_val.clone()),
                ));
            }
        }
        
        // Check for removed keys
        for (key, old_val) in old_obj {
            if !new_obj.contains_key(key) {
                changes.push(FieldChange::new(
                    key,
                    Some(old_val.clone()),
                    None,
                ));
            }
        }
        
        changes
    }
}

/// Compare two optional JSON values semantically.
/// Handles None/Null equivalence.
fn values_equal(old: Option<&serde_json::Value>, new: &serde_json::Value) -> bool {
    match old {
        None => new.is_null(),
        Some(a) => {
            if a.is_null() && new.is_null() { return true; }
            a == new
        }
    }
}
