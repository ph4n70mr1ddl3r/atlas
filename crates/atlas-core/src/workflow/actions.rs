//! Workflow Actions
//! 
//! Actions that execute when transitions occur or states are entered/exited.

use atlas_shared::{ActionDefinition, RecordId, AtlasError, AtlasResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tracing::{info, warn, debug};

/// Result of action execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub success: bool,
    pub action_name: String,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl ActionResult {
    pub fn success(action_name: &str, output: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            action_name: action_name.to_string(),
            output,
            error: None,
        }
    }
    
    pub fn failure(action_name: &str, error: String) -> Self {
        Self {
            success: false,
            action_name: action_name.to_string(),
            output: None,
            error: Some(error),
        }
    }
}

/// Action executor for workflow actions
pub struct ActionExecutor {
    handlers: Arc<RwLock<HashMap<String, ActionHandler>>>,
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl ActionExecutor {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            event_publisher: None,
        }
    }
    
    pub fn with_event_publisher(mut self, publisher: Arc<dyn EventPublisher>) -> Self {
        self.event_publisher = Some(publisher);
        self
    }
    
    /// Register a custom action handler
    pub async fn register_handler(&self, name: &str, handler: ActionHandler) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(name.to_string(), handler);
    }
    
    /// Execute an action
    pub async fn execute(
        &self,
        action_def: &ActionDefinition,
        record_id: RecordId,
        record_data: &serde_json::Value,
    ) -> AtlasResult<ActionResult> {
        match action_def {
            ActionDefinition::SetField { field, value } => {
                Ok(self.execute_set_field(field, value))
            }
            ActionDefinition::SendNotification { template, recipients } => {
                self.execute_send_notification(template, recipients, record_id).await
            }
            ActionDefinition::InvokeWebhook { url, method } => {
                self.execute_webhook(url, method, record_id, record_data).await
            }
            ActionDefinition::InvokeAction { service, action } => {
                self.execute_service_action(service, action, record_id, record_data).await
            }
            ActionDefinition::AssignRole { role, user_field } => {
                self.execute_assign_role(role, user_field, record_data).await
            }
            ActionDefinition::UpdateRelated { entity, filter, changes } => {
                self.execute_update_related(entity, filter, changes).await
            }
            ActionDefinition::CreateRecord { entity, values } => {
                self.execute_create_record(entity, values).await
            }
        }
    }
    
    fn execute_set_field(&self, field: &str, value: &serde_json::Value) -> ActionResult {
        info!("SetField action: {} = {}", field, value);
        // This would normally modify the record, but that's handled by the caller
        ActionResult::success("set_field", Some(serde_json::json!({
            "field": field,
            "value": value
        })))
    }
    
    async fn execute_send_notification(
        &self,
        template: &str,
        recipients: &Option<String>,
        record_id: RecordId,
    ) -> AtlasResult<ActionResult> {
        info!("SendNotification: template={}, record_id={}", template, record_id);
        
        // Publish notification event
        if let Some(publisher) = &self.event_publisher {
            let payload = serde_json::json!({
                "type": "notification",
                "template": template,
                "recipients": recipients,
                "record_id": record_id.to_string(),
                "context": {}
            });
            publisher.publish("atlas.notifications", &payload).await?;
        }
        
        Ok(ActionResult::success("send_notification", Some(serde_json::json!({
            "template": template,
            "recipients": recipients
        }))))
    }
    
    async fn execute_webhook(
        &self,
        url: &str,
        method: &str,
        record_id: RecordId,
        data: &serde_json::Value,
    ) -> AtlasResult<ActionResult> {
        debug!("InvokeWebhook: {} {} for record {}", method, url, record_id);
        
        // In a real implementation, this would make an HTTP request
        // For now, we just log and return success
        Ok(ActionResult::success("invoke_webhook", Some(serde_json::json!({
            "url": url,
            "method": method,
            "record_id": record_id.to_string()
        }))))
    }
    
    async fn execute_service_action(
        &self,
        service: &str,
        action: &str,
        record_id: RecordId,
        data: &serde_json::Value,
    ) -> AtlasResult<ActionResult> {
        info!("InvokeAction: {}.{} for record {}", service, action, record_id);
        
        // Publish event for the target service
        if let Some(publisher) = &self.event_publisher {
            let payload = serde_json::json!({
                "type": "service_action",
                "service": service,
                "action": action,
                "record_id": record_id.to_string(),
                "data": data
            });
            publisher.publish(&format!("atlas.services.{}.action", service), &payload).await?;
        }
        
        Ok(ActionResult::success("invoke_action", Some(serde_json::json!({
            "service": service,
            "action": action
        }))))
    }
    
    async fn execute_assign_role(
        &self,
        role: &str,
        user_field: &Option<String>,
        data: &serde_json::Value,
    ) -> AtlasResult<ActionResult> {
        info!("AssignRole: {} to user from field {:?}", role, user_field);
        
        // If user_field is specified, get the user ID from the record
        let user_id = if let Some(field) = user_field {
            data.get(field).and_then(|v| v.as_str()).map(|s| s.to_string())
        } else {
            None
        };
        
        if let Some(publisher) = &self.event_publisher {
            let payload = serde_json::json!({
                "type": "assign_role",
                "role": role,
                "user_id": user_id,
                "data": data
            });
            publisher.publish("atlas.auth.role_assignment", &payload).await?;
        }
        
        Ok(ActionResult::success("assign_role", Some(serde_json::json!({
            "role": role,
            "user_id": user_id
        }))))
    }
    
    async fn execute_update_related(
        &self,
        entity: &str,
        filter: &str,
        changes: &serde_json::Value,
    ) -> AtlasResult<ActionResult> {
        info!("UpdateRelated: {} with filter {}", entity, filter);
        
        if let Some(publisher) = &self.event_publisher {
            let payload = serde_json::json!({
                "type": "update_related",
                "entity": entity,
                "filter": filter,
                "changes": changes
            });
            publisher.publish(&format!("atlas.entity.{}.bulk_update", entity), &payload).await?;
        }
        
        Ok(ActionResult::success("update_related", Some(serde_json::json!({
            "entity": entity,
            "changes": changes
        }))))
    }
    
    async fn execute_create_record(
        &self,
        entity: &str,
        values: &serde_json::Value,
    ) -> AtlasResult<ActionResult> {
        info!("CreateRecord: {} with values {:?}", entity, values);
        
        if let Some(publisher) = &self.event_publisher {
            let payload = serde_json::json!({
                "type": "create_record",
                "entity": entity,
                "values": values
            });
            publisher.publish(&format!("atlas.entity.{}.create", entity), &payload).await?;
        }
        
        Ok(ActionResult::success("create_record", Some(serde_json::json!({
            "entity": entity
        }))))
    }
}

impl Default for ActionExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom action handler function type
pub type ActionHandler = Arc<dyn Fn(RecordId, &serde_json::Value) -> Box<dyn std::future::Future<Output = AtlasResult<ActionResult>> + Send> + Send + Sync>;

/// Event publisher trait
#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, subject: &str, payload: &serde_json::Value) -> AtlasResult<()>;
}

/// No-op event publisher for testing
pub struct NoOpEventPublisher;

#[async_trait::async_trait]
impl EventPublisher for NoOpEventPublisher {
    async fn publish(&self, _subject: &str, _payload: &serde_json::Value) -> AtlasResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_set_field_action() {
        let executor = ActionExecutor::new();
        
        let action = ActionDefinition::SetField {
            field: "status".to_string(),
            value: serde_json::json!("approved"),
        };
        
        let result = executor.execute(
            &action,
            uuid::Uuid::new_v4(),
            &serde_json::json!({}),
        ).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.action_name, "set_field");
    }
    
    #[tokio::test]
    async fn test_create_record_action() {
        let executor = ActionExecutor::new();
        
        let action = ActionDefinition::CreateRecord {
            entity: "audit_log".to_string(),
            values: serde_json::json!({
                "action": "approved",
                "timestamp": "2024-01-01"
            }),
        };
        
        let result = executor.execute(
            &action,
            uuid::Uuid::new_v4(),
            &serde_json::json!({}),
        ).await.unwrap();
        
        assert!(result.success);
    }
}
