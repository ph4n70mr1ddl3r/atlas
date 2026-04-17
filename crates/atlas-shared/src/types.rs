//! Atlas Core Types
//! 
//! These types define the declarative foundation of Atlas.
//! Everything is data that can be configured and reloaded at runtime.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Unique identifier types
pub type EntityId = Uuid;
pub type FieldId = Uuid;
pub type RecordId = Uuid;
pub type OrganizationId = Uuid;
pub type UserId = Uuid;
pub type SessionId = Uuid;

/// Represents the definition of a field type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldType {
    /// Variable-length string with optional max length
    String {
        max_length: Option<usize>,
        pattern: Option<String>,
    },
    /// Fixed string (UUIDs, codes)
    FixedString {
        length: usize,
    },
    /// Integer with optional bounds
    Integer {
        min: Option<i64>,
        max: Option<i64>,
    },
    /// Decimal number with precision
    Decimal {
        precision: u8,
        scale: u8,
    },
    /// Boolean true/false
    Boolean,
    /// Date only
    Date,
    /// Date with time
    DateTime,
    /// Reference to another entity
    Reference {
        entity: String,
        field: Option<String>,
    },
    /// One-to-many relationship
    OneToMany {
        entity: String,
        foreign_key: String,
    },
    /// One-to-one relationship
    OneToOne {
        entity: String,
        foreign_key: String,
    },
    /// Enum with allowed values
    Enum {
        values: Vec<String>,
    },
    /// Computed field with formula
    Computed {
        formula: String,
        return_type: Box<FieldType>,
    },
    /// File attachment
    Attachment,
    /// Currency with code
    Currency {
        code: String,
    },
    /// Rich HTML text
    RichText,
    /// Flexible JSON
    Json,
    /// Email address
    Email,
    /// URL
    Url,
    /// Phone number
    Phone,
    /// Location/Address
    Address,
}

impl Default for FieldType {
    fn default() -> Self {
        FieldType::String { max_length: None, pattern: None }
    }
}

/// Field definition within an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldDefinition {
    pub id: Option<FieldId>,
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub is_required: bool,
    pub is_unique: bool,
    pub is_read_only: bool,
    pub is_searchable: bool,
    pub default_value: Option<serde_json::Value>,
    pub help_text: Option<String>,
    pub display_order: i32,
    pub placeholder: Option<String>,
    pub validations: Vec<ValidationRule>,
    pub visibility: VisibilityRule,
    pub formatting: Option<FormatRule>,
}

impl FieldDefinition {
    pub fn new(name: &str, label: &str, field_type: FieldType) -> Self {
        Self {
            id: None,
            name: name.to_string(),
            label: label.to_string(),
            field_type,
            is_required: false,
            is_unique: false,
            is_read_only: false,
            is_searchable: true,
            default_value: None,
            help_text: None,
            display_order: 0,
            placeholder: None,
            validations: vec![],
            visibility: VisibilityRule::default(),
            formatting: None,
        }
    }
}

/// Validation rules for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ValidationRule {
    Required,
    MinLength { value: usize },
    MaxLength { value: usize },
    Min { value: f64 },
    Max { value: f64 },
    Pattern { value: String },
    Email,
    Url,
    Custom { expression: String, message: String },
}

/// Rules for field visibility
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct VisibilityRule {
    pub condition: Option<String>,
    pub roles: Vec<String>,
    pub hidden: bool,
}

/// Formatting rules for display
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatRule {
    pub format: Option<String>,
    pub currency_code: Option<String>,
    pub decimals: Option<u8>,
}

/// Entity definition (the schema for a business object)
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EntityDefinition {
    #[serde(default)]
    pub id: Option<EntityId>,
    pub name: String,
    pub label: String,
    pub plural_label: String,
    pub table_name: Option<String>,
    pub description: Option<String>,
    
    #[serde(default)]
    pub fields: Vec<FieldDefinition>,
    
    #[serde(default)]
    pub indexes: Vec<IndexDefinition>,
    
    #[serde(default)]
    pub workflow: Option<WorkflowDefinition>,
    
    #[serde(default)]
    pub security: Option<SecurityPolicy>,
    
    // Audit settings
    #[serde(default = "default_true")]
    pub is_audit_enabled: bool,
    
    #[serde(default = "default_true")]
    pub is_soft_delete: bool,
    
    #[serde(default)]
    pub icon: Option<String>,
    
    #[serde(default)]
    pub color: Option<String>,
    
    #[serde(default)]
    pub metadata: serde_json::Value,
}

fn default_true() -> bool { true }

/// Index definition for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexDefinition {
    pub name: String,
    pub fields: Vec<String>,
    pub is_unique: bool,
}

/// Workflow definition for an entity
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDefinition {
    #[serde(default)]
    pub id: Option<Uuid>,
    pub name: String,
    pub initial_state: String,
    
    #[validate(nested)]
    pub states: Vec<StateDefinition>,
    
    #[validate(nested)]
    pub transitions: Vec<TransitionDefinition>,
    
    #[serde(default = "default_true")]
    pub is_active: bool,
}

/// State within a workflow
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct StateDefinition {
    pub name: String,
    pub label: String,
    pub state_type: StateType,
    
    #[serde(default)]
    pub entry_actions: Vec<ActionDefinition>,
    
    #[serde(default)]
    pub exit_actions: Vec<ActionDefinition>,
    
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum StateType {
    Initial,
    Working,
    Final,
}

/// Transition between workflow states
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct TransitionDefinition {
    pub name: String,
    pub from_state: String,
    pub to_state: String,
    pub action: String,
    pub action_label: Option<String>,
    
    #[serde(default)]
    pub guards: Vec<GuardDefinition>,
    
    #[serde(default)]
    pub required_roles: Vec<String>,
    
    #[serde(default)]
    pub entry_actions: Vec<ActionDefinition>,
    
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Guard conditions for transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GuardDefinition {
    Validate { rule: String },
    Expression { expression: String },
    Role { roles: Vec<String> },
    Custom { handler: String },
}

/// Action definitions (what happens on transitions or state entry/exit)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionDefinition {
    SetField { field: String, value: serde_json::Value },
    SendNotification { template: String, recipients: Option<String> },
    InvokeWebhook { url: String, method: String },
    InvokeAction { service: String, action: String },
    AssignRole { role: String, user_field: Option<String> },
    UpdateRelated { entity: String, filter: String, changes: serde_json::Value },
    CreateRecord { entity: String, values: serde_json::Value },
}

impl ActionDefinition {
    /// Returns a handler lookup key if this action maps to a registered handler.
    /// `InvokeAction` uses "service.action" as the key; other variants return None.
    pub fn handler_name(&self) -> Option<&str> {
        match self {
            ActionDefinition::InvokeAction { service: _, action } => {
                // Return a combined key; the caller can split on '.' if needed.
                // For now we just return the action name which is what handlers
                // are registered under.
                Some(action.as_str())
            }
            _ => None,
        }
    }
}

/// Security policy for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityPolicy {
    pub name: String,
    #[serde(default)]
    pub rules: Vec<SecurityRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "effect", rename_all = "snake_case")]
pub enum SecurityRule {
    Allow { actions: Vec<String>, condition: Option<String> },
    Deny { actions: Vec<String>, condition: Option<String> },
}

// ============================================================================
// Record Types (Runtime Data)
// ============================================================================

/// A single record (row) of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: RecordId,
    pub entity_name: String,
    pub values: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<UserId>,
    pub updated_by: Option<UserId>,
}

/// Query filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryFilter {
    #[serde(default)]
    pub field: String,
    #[serde(default)]
    pub operator: FilterOperator,
    #[serde(default)]
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum FilterOperator {
    #[default]
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    NotIn,
    Contains,
    StartsWith,
    EndsWith,
    IsNull,
    IsNotNull,
    Between,
}

/// Query request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    pub entity: String,
    #[serde(default)]
    pub filters: Vec<QueryFilter>,
    #[serde(default)]
    pub sort_by: Vec<SortOrder>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    #[serde(default)]
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortOrder {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Query response with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse<T> {
    pub data: Vec<T>,
    pub meta: QueryMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryMeta {
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
    pub has_more: bool,
}

/// Create request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub entity: String,
    pub values: serde_json::Value,
}

/// Update request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRequest {
    pub entity: String,
    pub id: RecordId,
    pub values: serde_json::Value,
}

/// Workflow action request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowActionRequest {
    pub action: String,
    pub comment: Option<String>,
    pub values: Option<serde_json::Value>,
}

// ============================================================================
// Organization & User Context
// ============================================================================

/// Organization context (for multi-tenancy)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub id: OrganizationId,
    pub name: String,
    pub code: String,
    pub parent_id: Option<OrganizationId>,
    pub metadata: serde_json::Value,
}

/// User context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: String,
    pub organization_id: OrganizationId,
    pub roles: Vec<String>,
    pub is_active: bool,
}

/// Session context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub organization_id: OrganizationId,
    pub roles: Vec<String>,
    pub expires_at: DateTime<Utc>,
}

// ============================================================================
// Audit Trail
// ============================================================================

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: RecordId,
    pub action: AuditAction,
    pub old_data: Option<serde_json::Value>,
    pub new_data: Option<serde_json::Value>,
    pub changed_by: Option<UserId>,
    pub changed_at: DateTime<Utc>,
    pub session_id: Option<SessionId>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    ExecuteAction,
    Login,
    Logout,
}

// ============================================================================
// API Response Types
// ============================================================================

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ApiResponse<T> {
    Success { data: T },
    Error(ApiError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

// ============================================================================
// Schema Metadata
// ============================================================================

/// Schema version for caching and hot-reload
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVersion {
    pub entity: String,
    pub version: i64,
    pub updated_at: DateTime<Utc>,
}

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
// Import Job Types
// ============================================================================

/// Import job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ImportJobStatus {
    Pending,
    Validating,
    Importing,
    Completed,
    Failed,
    Cancelled,
}

/// Import job tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportJob {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub entity_type: String,
    pub format: String,
    pub status: String,
    pub total_rows: i32,
    pub processed_rows: i32,
    pub imported_rows: i32,
    pub failed_rows: i32,
    pub skipped_rows: i32,
    pub original_filename: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub field_mapping: serde_json::Value,
    pub upsert_mode: bool,
    pub skip_validation: bool,
    pub stop_on_error: bool,
    pub validation_errors: serde_json::Value,
    pub import_errors: serde_json::Value,
    pub duplicate_action: String,
    pub duplicates_found: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Duplicate Detection
// ============================================================================

/// Duplicate detection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub description: Option<String>,
    pub match_criteria: serde_json::Value,
    pub filter_condition: serde_json::Value,
    pub on_duplicate: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Duplicate match criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchCriterion {
    pub field: String,
    pub match_type: String, // "exact", "fuzzy", "case_insensitive"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
}

/// Detected duplicate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedDuplicate {
    pub rule_name: String,
    pub entity_type: String,
    pub existing_record_id: Uuid,
    pub match_field: String,
    pub match_type: String,
    pub existing_value: serde_json::Value,
    pub new_value: serde_json::Value,
}
