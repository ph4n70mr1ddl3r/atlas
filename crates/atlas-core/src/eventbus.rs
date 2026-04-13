//! Event Bus Integration
//! 
//! NATS-based event bus for inter-service communication.

use atlas_shared::events::{AtlasEvent, EventPayload, subjects};
use atlas_shared::errors::{AtlasError, AtlasResult};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn, error, debug};

/// Event bus trait for publishing and subscribing to events
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish an event
    async fn publish(&self, event: AtlasEvent) -> AtlasResult<()>;
    
    /// Publish an event to a specific subject
    async fn publish_to(&self, subject: &str, event: AtlasEvent) -> AtlasResult<()>;
    
    /// Check if connected
    fn is_connected(&self) -> bool;
}

/// NATS-based event bus implementation
pub struct NatsEventBus {
    client: Option<nats::asynk::Connection>,
    service_name: String,
}

impl NatsEventBus {
    /// Create a new NATS event bus
    pub async fn new(nats_url: &str, service_name: &str) -> AtlasResult<Self> {
        match nats::asynk::connect(nats_url).await {
            Ok(client) => {
                info!("Connected to NATS at {}", nats_url);
                Ok(Self {
                    client: Some(client),
                    service_name: service_name.to_string(),
                })
            }
            Err(e) => {
                warn!("Failed to connect to NATS: {}. Running without event bus.", e);
                Ok(Self {
                    client: None,
                    service_name: service_name.to_string(),
                })
            }
        }
    }
    
    /// Create a no-op event bus (for testing/dev without NATS)
    pub fn noop(service_name: &str) -> Self {
        Self {
            client: None,
            service_name: service_name.to_string(),
        }
    }
}

#[async_trait]
impl EventBus for NatsEventBus {
    async fn publish(&self, event: AtlasEvent) -> AtlasResult<()> {
        let subject = match &event.payload {
            EventPayload::ConfigChanged(p) => subjects::CONFIG_CHANGED.to_string(),
            EventPayload::RecordCreated(p) => subjects::entity_created(&p.entity_name),
            EventPayload::RecordUpdated(p) => subjects::entity_updated(&p.entity_name),
            EventPayload::RecordDeleted(p) => subjects::entity_deleted(&p.entity_name),
            EventPayload::WorkflowTransition(p) => subjects::workflow_transition(&p.entity_name),
            EventPayload::ServiceStarted(p) => subjects::SERVICE_HEALTH.to_string(),
            EventPayload::HealthCheck(p) => subjects::SERVICE_HEALTH.to_string(),
            _ => "atlas.events".to_string(),
        };
        
        self.publish_to(&subject, event).await
    }
    
    async fn publish_to(&self, subject: &str, event: AtlasEvent) -> AtlasResult<()> {
        if let Some(client) = &self.client {
            let payload = serde_json::to_string(&event)
                .map_err(|e| AtlasError::EventBusError(e.to_string()))?;
            
            client.publish(subject, &payload)
                .await
                .map_err(|e| AtlasError::EventBusError(e.to_string()))?;
            
            debug!("Published event to {}: {:?}", subject, event.event_type);
        } else {
            debug!("Event bus not connected, skipping publish to {}", subject);
        }
        
        Ok(())
    }
    
    fn is_connected(&self) -> bool {
        self.client.is_some()
    }
}

/// Helper for creating common events
pub struct EventFactory;

impl EventFactory {
    /// Create a record created event
    pub fn record_created(
        service: &str,
        entity_name: &str,
        record_id: atlas_shared::RecordId,
        data: serde_json::Value,
        user_id: Option<atlas_shared::UserId>,
    ) -> AtlasEvent {
        AtlasEvent::new(service, EventPayload::RecordCreated(
            atlas_shared::events::RecordPayload {
                entity_name: entity_name.to_string(),
                record_id,
                data,
                changed_by: user_id,
            }
        ))
    }
    
    /// Create a record updated event
    pub fn record_updated(
        service: &str,
        entity_name: &str,
        record_id: atlas_shared::RecordId,
        data: serde_json::Value,
        user_id: Option<atlas_shared::UserId>,
    ) -> AtlasEvent {
        AtlasEvent::new(service, EventPayload::RecordUpdated(
            atlas_shared::events::RecordPayload {
                entity_name: entity_name.to_string(),
                record_id,
                data,
                changed_by: user_id,
            }
        ))
    }
    
    /// Create a record deleted event
    pub fn record_deleted(
        service: &str,
        entity_name: &str,
        record_id: atlas_shared::RecordId,
        user_id: Option<atlas_shared::UserId>,
    ) -> AtlasEvent {
        AtlasEvent::new(service, EventPayload::RecordDeleted(
            atlas_shared::events::RecordDeletedPayload {
                entity_name: entity_name.to_string(),
                record_id,
                changed_by: user_id,
            }
        ))
    }
    
    /// Create a workflow transition event
    pub fn workflow_transition(
        service: &str,
        entity_name: &str,
        record_id: atlas_shared::RecordId,
        workflow_name: &str,
        from_state: &str,
        to_state: &str,
        action: &str,
        performed_by: Option<atlas_shared::UserId>,
    ) -> AtlasEvent {
        AtlasEvent::new(service, EventPayload::WorkflowTransition(
            atlas_shared::events::WorkflowTransitionPayload {
                entity_name: entity_name.to_string(),
                record_id,
                workflow_name: workflow_name.to_string(),
                from_state: from_state.to_string(),
                to_state: to_state.to_string(),
                action: action.to_string(),
                performed_by,
                comment: None,
            }
        ))
    }
    
    /// Create a config changed event
    pub fn config_changed(
        service: &str,
        config_type: &str,
        config_name: &str,
        version: i64,
        changes: Vec<String>,
    ) -> AtlasEvent {
        AtlasEvent::new(service, EventPayload::ConfigChanged(
            atlas_shared::events::ConfigChangedPayload {
                config_type: config_type.to_string(),
                config_name: config_name.to_string(),
                version,
                changes,
            }
        ))
    }
    
    /// Create a service started event
    pub fn service_started(
        service: &str,
        version: &str,
        host: &str,
        port: u16,
    ) -> AtlasEvent {
        AtlasEvent::new(service, EventPayload::ServiceStarted(
            atlas_shared::events::ServiceInfoPayload {
                service_name: service.to_string(),
                version: version.to_string(),
                host: host.to_string(),
                port,
            }
        ))
    }
}
