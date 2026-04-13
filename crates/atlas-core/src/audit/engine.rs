//! Audit Engine Implementation

use atlas_shared::{AuditEntry, AuditAction, RecordId, UserId, AtlasError, AtlasResult};
use super::{AuditQuery, AuditSummary, ChangeSet, FieldChange};
use super::AuditRepository;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, debug};

/// Audit engine for change tracking
pub struct AuditEngine {
    repository: Arc<dyn AuditRepository>,
    enabled_entities: std::sync::RwLock<std::collections::HashSet<String>>,
}

impl AuditEngine {
    pub fn new(repository: Arc<dyn AuditRepository>) -> Self {
        Self {
            repository,
            enabled_entities: std::sync::RwLock::new(std::collections::HashSet::new()),
        }
    }
    
    /// Enable auditing for an entity
    pub fn enable_audit(&self, entity: &str) {
        let mut entities = self.enabled_entities.write().unwrap();
        entities.insert(entity.to_string());
        info!("Audit enabled for entity: {}", entity);
    }
    
    /// Disable auditing for an entity
    pub fn disable_audit(&self, entity: &str) {
        let mut entities = self.enabled_entities.write().unwrap();
        entities.remove(entity);
        info!("Audit disabled for entity: {}", entity);
    }
    
    /// Check if auditing is enabled for an entity
    pub fn is_enabled(&self, entity: &str) -> bool {
        let entities = self.enabled_entities.read().unwrap();
        entities.contains(entity) || entities.is_empty() // Empty means all enabled
    }
    
    /// Log an audit entry
    pub async fn log(
        &self,
        entity_type: &str,
        entity_id: RecordId,
        action: AuditAction,
        old_data: Option<&serde_json::Value>,
        new_data: Option<&serde_json::Value>,
        user_id: Option<UserId>,
        session_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> AtlasResult<Uuid> {
        if !self.is_enabled(entity_type) {
            debug!("Audit skipped for {} (not enabled)", entity_type);
            return Ok(Uuid::new_v4()); // Return a fake ID
        }
        
        let entry = AuditEntry {
            id: Uuid::new_v4(),
            entity_type: entity_type.to_string(),
            entity_id,
            action,
            old_data: old_data.cloned(),
            new_data: new_data.cloned(),
            changed_by: user_id,
            changed_at: Utc::now(),
            session_id,
            ip_address: ip_address.map(|s| s.to_string()),
            user_agent: user_agent.map(|s| s.to_string()),
        };
        
        self.repository.insert(&entry).await?;
        
        Ok(entry.id)
    }
    
    /// Log a create action
    pub async fn log_create(
        &self,
        entity_type: &str,
        entity_id: RecordId,
        data: &serde_json::Value,
        user_id: Option<UserId>,
    ) -> AtlasResult<Uuid> {
        self.log(
            entity_type,
            entity_id,
            AuditAction::Create,
            None,
            Some(data),
            user_id,
            None,
            None,
            None,
        ).await
    }
    
    /// Log an update action
    pub async fn log_update(
        &self,
        entity_type: &str,
        entity_id: RecordId,
        old_data: &serde_json::Value,
        new_data: &serde_json::Value,
        user_id: Option<UserId>,
    ) -> AtlasResult<Uuid> {
        self.log(
            entity_type,
            entity_id,
            AuditAction::Update,
            Some(old_data),
            Some(new_data),
            user_id,
            None,
            None,
            None,
        ).await
    }
    
    /// Log a delete action
    pub async fn log_delete(
        &self,
        entity_type: &str,
        entity_id: RecordId,
        old_data: &serde_json::Value,
        user_id: Option<UserId>,
    ) -> AtlasResult<Uuid> {
        self.log(
            entity_type,
            entity_id,
            AuditAction::Delete,
            Some(old_data),
            None,
            user_id,
            None,
            None,
            None,
        ).await
    }
    
    /// Get audit trail for an entity
    pub async fn get_entity_history(&self, entity_type: &str, entity_id: RecordId) -> AtlasResult<Vec<AuditEntry>> {
        self.repository.query(&AuditQuery {
            entity_type: Some(entity_type.to_string()),
            entity_id: Some(entity_id),
            action: None,
            user_id: None,
            from_date: None,
            to_date: None,
            limit: None,
            offset: None,
        }).await
    }
    
    /// Get changes between two audit entries
    pub async fn get_changes(&self, entry1_id: Uuid, entry2_id: Uuid) -> AtlasResult<ChangeSet> {
        let entries = self.repository.get_by_ids(&[entry1_id, entry2_id]).await?;
        
        if entries.len() != 2 {
            return Err(AtlasError::Internal("Expected 2 entries".to_string()).into());
        }
        
        let (entry1, entry2) = if entries[0].id == entry1_id {
            (&entries[0], &entries[1])
        } else {
            (&entries[1], &entries[0])
        };
        
        let changes = FieldChange::compute_changes(
            entry1.new_data.as_ref().unwrap_or(&serde_json::Value::Null),
            entry2.new_data.as_ref().unwrap_or(&serde_json::Value::Null),
        );
        
        Ok(ChangeSet {
            entity_id: entry1.entity_id,
            changes,
        })
    }
    
    /// Query audit entries
    pub async fn query(&self, query: &AuditQuery) -> AtlasResult<Vec<AuditEntry>> {
        self.repository.query(query).await
    }
    
    /// Get audit summary
    pub async fn get_summary(&self, entity_type: Option<&str>, days: i64) -> AtlasResult<AuditSummary> {
        let from_date = Utc::now() - chrono::Duration::days(days);
        
        let entries = self.repository.query(&AuditQuery {
            entity_type: entity_type.map(|s| s.to_string()),
            entity_id: None,
            action: None,
            user_id: None,
            from_date: Some(from_date),
            to_date: Some(Utc::now()),
            limit: Some(1000),
            offset: None,
        }).await?;
        
        let mut actions_by_type: HashMap<String, i64> = HashMap::new();
        let mut actions_by_user: HashMap<String, i64> = HashMap::new();
        
        for entry in &entries {
            let action_key = format!("{:?}", entry.action);
            *actions_by_type.entry(action_key).or_insert(0) += 1;
            
            if let Some(user_id) = entry.changed_by {
                let user_key = user_id.to_string();
                *actions_by_user.entry(user_key).or_insert(0) += 1;
            }
        }
        
        Ok(AuditSummary {
            total_actions: entries.len() as i64,
            actions_by_type,
            actions_by_user,
            recent_activity: entries,
        })
    }
}
