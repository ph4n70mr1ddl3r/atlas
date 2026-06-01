use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
// ============================================================================
// Notification Types (Oracle Fusion bell icon notifications)
// ============================================================================

/// Notification priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum NotificationPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

/// Notification type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    WorkflowAction,
    ApprovalRequired,
    Escalation,
    System,
    Mention,
    Assignment,
    DuplicateDetected,
    ImportCompleted,
}

/// Notification record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub notification_type: String,
    pub priority: String,
    pub title: String,
    pub message: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub action_url: Option<String>,
    pub workflow_name: Option<String>,
    pub from_state: Option<String>,
    pub to_state: Option<String>,
    pub action: Option<String>,
    pub performed_by: Option<Uuid>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub is_dismissed: bool,
    pub dismissed_at: Option<DateTime<Utc>>,
    pub channels: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Create notification request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateNotificationRequest {
    pub user_id: Option<Uuid>,
    pub role: Option<String>,
    pub notification_type: String,
    pub priority: Option<String>,
    pub title: String,
    pub message: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub action_url: Option<String>,
    pub workflow_name: Option<String>,
    pub from_state: Option<String>,
    pub to_state: Option<String>,
    pub action: Option<String>,
    pub performed_by: Option<Uuid>,
    pub channels: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// Saved Searches (Oracle Fusion personalized views)
// ============================================================================

/// Saved search / personalized view
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSearch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
    pub filters: serde_json::Value,
    pub sort_by: String,
    pub sort_direction: String,
    pub columns: serde_json::Value,
    pub columns_widths: serde_json::Value,
    pub page_size: i32,
    pub is_shared: bool,
    pub is_default: bool,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update saved search request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedSearchRequest {
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
    pub filters: Option<serde_json::Value>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub columns: Option<serde_json::Value>,
    pub columns_widths: Option<serde_json::Value>,
    pub page_size: Option<i32>,
    pub is_shared: Option<bool>,
    pub is_default: Option<bool>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

// ============================================================================
// Approval Chains (Oracle Fusion multi-level approvals)
// ============================================================================

/// Approval chain definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalChain {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
    pub condition_expression: Option<String>,
    pub chain_definition: serde_json::Value,
    pub escalation_enabled: bool,
    pub escalation_hours: i32,
    pub escalation_to_roles: serde_json::Value,
    pub allow_delegation: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Approval level within a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalLevel {
    pub level: i32,
    pub approver_type: String, // "role", "user", "auto"
    pub roles: Vec<String>,
    pub user_ids: Option<Vec<Uuid>>,
    pub auto_approve_after_hours: Option<i32>,
}

/// Approval request status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Escalated,
    Cancelled,
    TimedOut,
}

/// Approval request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub chain_id: Option<Uuid>,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub current_level: i32,
    pub total_levels: i32,
    pub status: String,
    pub requested_by: Uuid,
    pub requested_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub completed_by: Option<Uuid>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub steps: Vec<ApprovalStep>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual approval step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalStep {
    pub id: Uuid,
    pub approval_request_id: Uuid,
    pub level: i32,
    pub approver_type: String,
    pub approver_role: Option<String>,
    pub approver_user_id: Option<Uuid>,
    pub is_delegated: bool,
    pub delegated_by: Option<Uuid>,
    pub delegated_to: Option<Uuid>,
    pub status: String,
    pub action_at: Option<DateTime<Utc>>,
    pub action_by: Option<Uuid>,
    pub comment: Option<String>,
    pub auto_approve_after_hours: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Approval Delegation Rules (Oracle Fusion BPM Worklist > Rules > Delegation)
// ============================================================================

/// Approval delegation rule
/// Oracle Fusion: BPM Worklist > Rules > Configure Delegation
/// Users proactively set up delegation rules like:
/// "While I'm on vacation from June 1-15, delegate all my approvals to Jane Smith"
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalDelegationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub delegator_id: Uuid,
    pub delegate_to_id: Uuid,
    pub rule_name: String,
    pub description: Option<String>,
    /// 'all', '`by_category`', '`by_role`', '`by_entity`'
    pub delegation_type: String,
    /// For type '`by_category`': categories as JSON array
    pub categories: serde_json::Value,
    /// For type '`by_role`': roles as JSON array
    pub roles: serde_json::Value,
    /// For type '`by_entity`': entity types as JSON array
    pub entity_types: serde_json::Value,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub is_active: bool,
    pub auto_activate: bool,
    pub auto_expire: bool,
    /// 'scheduled', 'active', 'expired', 'cancelled'
    pub status: String,
    pub activated_at: Option<DateTime<Utc>>,
    pub expired_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<Uuid>,
    pub cancellation_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update delegation rule request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDelegationRuleRequest {
    pub delegate_to_id: String,
    pub rule_name: String,
    pub description: Option<String>,
    #[serde(default = "default_delegation_all")]
    pub delegation_type: String,
    #[serde(default)]
    pub categories: Option<serde_json::Value>,
    #[serde(default)]
    pub roles: Option<serde_json::Value>,
    #[serde(default)]
    pub entity_types: Option<serde_json::Value>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub auto_activate: Option<bool>,
    pub auto_expire: Option<bool>,
}

pub fn default_delegation_all() -> String {
    "all".to_string()
}

/// Delegation history entry (tracks when delegations were actually used)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegationHistoryEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub delegation_rule_id: Uuid,
    pub original_approver_id: Uuid,
    pub delegated_to_id: Uuid,
    pub approval_step_id: Option<Uuid>,
    pub approval_request_id: Option<Uuid>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub action_taken: Option<String>,
    pub action_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Delegation dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegationDashboard {
    pub total_active_rules: i64,
    pub total_scheduled_rules: i64,
    pub total_expired_rules: i64,
    pub total_delegations_today: i64,
    pub delegations_by_type: serde_json::Value,
    pub recent_delegations: Vec<DelegationHistoryEntry>,
}
