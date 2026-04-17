//! Atlas Event Types
//! 
//! Events published to NATS for inter-service communication.
//! These events enable loose coupling between microservices.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{RecordId, UserId, OrganizationId};

/// Event envelope for all Atlas events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasEvent {
    pub id: Uuid,
    pub event_type: EventType,
    pub source_service: String,
    pub organization_id: Option<OrganizationId>,
    pub timestamp: DateTime<Utc>,
    pub payload: EventPayload,
}

impl AtlasEvent {
    pub fn new(source_service: &str, payload: EventPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: payload.event_type(),
            source_service: source_service.to_string(),
            organization_id: None,
            timestamp: Utc::now(),
            payload,
        }
    }
    
    pub fn with_org(mut self, org_id: OrganizationId) -> Self {
        self.organization_id = Some(org_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // Schema events
    EntityCreated,
    EntityUpdated,
    EntityDeleted,
    FieldCreated,
    FieldUpdated,
    FieldDeleted,
    
    // Data events
    RecordCreated,
    RecordUpdated,
    RecordDeleted,
    
    // Workflow events
    WorkflowTransition,
    WorkflowStateEntered,
    WorkflowStateExited,
    
    // Approval events (Oracle Fusion)
    ApprovalRequested,
    ApprovalStepCompleted,
    ApprovalDelegated,
    ApprovalEscalated,
    
    // Notification events
    NotificationCreated,
    NotificationRead,
    
    // Duplicate detection event
    DuplicateDetected,
    
    // Import event
    ImportCompleted,
    ImportFailed,
    
    // Config events
    ConfigChanged,
    ConfigReloaded,
    
    // Auth events
    UserLoggedIn,
    UserLoggedOut,
    UserCreated,
    UserUpdated,
    
    // System events
    ServiceStarted,
    ServiceStopped,
    HealthCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    // Schema payloads
    EntityCreated(EntityCreatedPayload),
    EntityUpdated(EntityUpdatedPayload),
    FieldCreated(FieldCreatedPayload),
    FieldUpdated(FieldUpdatedPayload),
    
    // Data payloads
    RecordCreated(RecordPayload),
    RecordUpdated(RecordPayload),
    RecordDeleted(RecordDeletedPayload),
    
    // Workflow payloads
    WorkflowTransition(WorkflowTransitionPayload),
    
    // Approval payloads (Oracle Fusion)
    ApprovalRequested(ApprovalEventPayload),
    ApprovalStepCompleted(ApprovalEventPayload),
    ApprovalDelegated(ApprovalDelegationPayload),
    ApprovalEscalated(ApprovalEventPayload),
    
    // Notification payloads
    NotificationCreated(NotificationEventPayload),
    
    // Duplicate detection
    DuplicateDetected(DuplicateDetectedPayload),
    
    // Import
    ImportCompleted(ImportEventPayload),
    ImportFailed(ImportEventPayload),
    
    // Config payloads
    ConfigChanged(ConfigChangedPayload),
    
    // Auth payloads
    UserLoggedIn(UserLoginPayload),
    UserLoggedOut(UserLogoutPayload),
    
    // System payloads
    ServiceStarted(ServiceInfoPayload),
    HealthCheck(HealthCheckPayload),
}

impl EventPayload {
    pub fn event_type(&self) -> EventType {
        match self {
            EventPayload::EntityCreated(_) => EventType::EntityCreated,
            EventPayload::EntityUpdated(_) => EventType::EntityUpdated,
            EventPayload::FieldCreated(_) => EventType::FieldCreated,
            EventPayload::FieldUpdated(_) => EventType::FieldUpdated,
            EventPayload::RecordCreated(_) => EventType::RecordCreated,
            EventPayload::RecordUpdated(_) => EventType::RecordUpdated,
            EventPayload::RecordDeleted(_) => EventType::RecordDeleted,
            EventPayload::WorkflowTransition(_) => EventType::WorkflowTransition,
            EventPayload::ApprovalRequested(_) => EventType::ApprovalRequested,
            EventPayload::ApprovalStepCompleted(_) => EventType::ApprovalStepCompleted,
            EventPayload::ApprovalDelegated(_) => EventType::ApprovalDelegated,
            EventPayload::ApprovalEscalated(_) => EventType::ApprovalEscalated,
            EventPayload::NotificationCreated(_) => EventType::NotificationCreated,
            EventPayload::DuplicateDetected(_) => EventType::DuplicateDetected,
            EventPayload::ImportCompleted(_) => EventType::ImportCompleted,
            EventPayload::ImportFailed(_) => EventType::ImportFailed,
            EventPayload::ConfigChanged(_) => EventType::ConfigChanged,
            EventPayload::UserLoggedIn(_) => EventType::UserLoggedIn,
            EventPayload::UserLoggedOut(_) => EventType::UserLoggedOut,
            EventPayload::ServiceStarted(_) => EventType::ServiceStarted,
            EventPayload::HealthCheck(_) => EventType::HealthCheck,
        }
    }
}

// ============================================================================
// Payload Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityCreatedPayload {
    pub entity_name: String,
    pub entity_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityUpdatedPayload {
    pub entity_name: String,
    pub entity_id: Uuid,
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldCreatedPayload {
    pub entity_name: String,
    pub field_name: String,
    pub field_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldUpdatedPayload {
    pub entity_name: String,
    pub field_name: String,
    pub field_id: Uuid,
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordPayload {
    pub entity_name: String,
    pub record_id: RecordId,
    pub data: serde_json::Value,
    pub changed_by: Option<UserId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDeletedPayload {
    pub entity_name: String,
    pub record_id: RecordId,
    pub changed_by: Option<UserId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTransitionPayload {
    pub entity_name: String,
    pub record_id: RecordId,
    pub workflow_name: String,
    pub from_state: String,
    pub to_state: String,
    pub action: String,
    pub performed_by: Option<UserId>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigChangedPayload {
    pub config_type: String,
    pub config_name: String,
    pub version: i64,
    pub changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLoginPayload {
    pub user_id: UserId,
    pub email: String,
    pub session_id: Uuid,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLogoutPayload {
    pub user_id: UserId,
    pub session_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfoPayload {
    pub service_name: String,
    pub version: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckPayload {
    pub service_name: String,
    pub status: HealthStatus,
    pub dependencies: Vec<DependencyStatus>,
}

// ============================================================================
// Approval Event Payloads (Oracle Fusion)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalEventPayload {
    pub approval_request_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub level: i32,
    pub approver_user_id: Option<Uuid>,
    pub approver_role: Option<String>,
    pub status: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalDelegationPayload {
    pub approval_request_id: Uuid,
    pub step_id: Uuid,
    pub delegated_from: Uuid,
    pub delegated_to: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
}

// ============================================================================
// Notification Event Payload
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationEventPayload {
    pub notification_id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub title: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
}

// ============================================================================
// Duplicate Detection & Import Event Payloads
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateDetectedPayload {
    pub entity_type: String,
    pub rule_name: String,
    pub existing_record_id: Uuid,
    pub new_record_data: serde_json::Value,
    pub match_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportEventPayload {
    pub job_id: Uuid,
    pub entity_type: String,
    pub total_rows: i32,
    pub imported_rows: i32,
    pub failed_rows: i32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyStatus {
    pub name: String,
    pub status: HealthStatus,
    pub latency_ms: Option<u64>,
}

// ============================================================================
// NATS Subjects
// ============================================================================

/// NATS subject naming conventions
pub mod subjects {
    /// Global subjects (broadcast to all)
    pub const CONFIG_CHANGED: &str = "atlas.config.changed";
    pub const SERVICE_HEALTH: &str = "atlas.health";
    pub const SCHEMA_CHANGED: &str = "atlas.schema.changed";
    
    /// Organization-scoped subjects
    pub fn org_events(org_id: &str) -> String {
        format!("atlas.org.{}.events", org_id)
    }
    
    /// Entity-specific subjects
    pub fn entity_created(entity: &str) -> String {
        format!("atlas.entity.{}.created", entity)
    }
    
    pub fn entity_updated(entity: &str) -> String {
        format!("atlas.entity.{}.updated", entity)
    }
    
    pub fn entity_deleted(entity: &str) -> String {
        format!("atlas.entity.{}.deleted", entity)
    }
    
    /// Workflow subjects
    pub fn workflow_transition(entity: &str) -> String {
        format!("atlas.workflow.{}.transition", entity)
    }
    
    /// Audit subjects
    pub fn audit_log() -> String {
        "atlas.audit".to_string()
    }
    
    /// Approval subjects
    pub fn approval_requested() -> String {
        "atlas.approval.requested".to_string()
    }
    
    pub fn approval_completed() -> String {
        "atlas.approval.completed".to_string()
    }
    
    pub fn approval_delegated() -> String {
        "atlas.approval.delegated".to_string()
    }
    
    pub fn approval_escalated() -> String {
        "atlas.approval.escalated".to_string()
    }
    
    /// Notification subjects
    pub fn notification_created() -> String {
        "atlas.notification.created".to_string()
    }
    
    /// Import status
    pub fn import_status() -> String {
        "atlas.import.status".to_string()
    }
    
    /// Duplicate detection
    pub fn duplicate_detected() -> String {
        "atlas.duplicate.detected".to_string()
    }
}
