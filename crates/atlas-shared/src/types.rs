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
// Period Close Management (Oracle Fusion General Ledger)
// ============================================================================

/// Accounting calendar definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingCalendar {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub calendar_type: String,
    pub fiscal_year_start_month: i32,
    pub periods_per_year: i32,
    pub has_adjusting_period: bool,
    pub current_fiscal_year: Option<i32>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// Create/update calendar request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingCalendarRequest {
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_monthly")]
    pub calendar_type: String,
    #[serde(default = "default_one")]
    pub fiscal_year_start_month: i32,
    #[serde(default = "default_twelve")]
    pub periods_per_year: i32,
    #[serde(default)]
    pub has_adjusting_period: bool,
    pub current_fiscal_year: Option<i32>,
}

fn default_monthly() -> String { "monthly".to_string() }
fn default_one() -> i32 { 1 }
fn default_twelve() -> i32 { 12 }

/// Period status within the financial close cycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PeriodStatus {
    #[default]
    NotOpened,
    Future,
    Open,
    PendingClose,
    Closed,
    PermanentlyClosed,
}

impl PeriodStatus {
    /// Whether posting is allowed in this period status
    pub fn allows_posting(&self) -> bool {
        matches!(self, PeriodStatus::Open | PeriodStatus::PendingClose)
    }

    /// Whether the status can be changed
    pub fn is_changeable(&self) -> bool {
        !matches!(self, PeriodStatus::PermanentlyClosed)
    }
}

/// Accounting period within a calendar
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingPeriod {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub calendar_id: Uuid,
    pub period_name: String,
    pub period_number: i32,
    pub fiscal_year: i32,
    pub quarter: Option<i32>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub status: String,
    pub status_changed_by: Option<Uuid>,
    pub status_changed_at: Option<DateTime<Utc>>,
    pub closed_by: Option<Uuid>,
    pub closed_at: Option<DateTime<Utc>>,
    pub period_type: String,
    // Subledger statuses
    pub gl_status: String,
    pub ap_status: String,
    pub ar_status: String,
    pub fa_status: String,
    pub po_status: String,
    // Aggregated balances (NUMERIC from DB, serialized as string/number)
    pub total_debits: serde_json::Value,
    pub total_credits: serde_json::Value,
    pub net_activity: serde_json::Value,
    pub beginning_balance: serde_json::Value,
    pub ending_balance: serde_json::Value,
    pub journal_entry_count: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Period close checklist item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodCloseChecklistItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub period_id: Uuid,
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_order: i32,
    pub category: Option<String>,
    pub subledger: Option<String>,
    pub status: String,
    pub assigned_to: Option<Uuid>,
    pub due_date: Option<chrono::NaiveDate>,
    pub completed_by: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub depends_on: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create checklist item request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChecklistItemRequest {
    pub task_name: String,
    pub task_description: Option<String>,
    pub task_order: Option<i32>,
    pub category: Option<String>,
    pub subledger: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub due_date: Option<chrono::NaiveDate>,
    pub depends_on: Option<Uuid>,
    pub notes: Option<String>,
}

/// Period close dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodCloseSummary {
    pub calendar_id: Uuid,
    pub calendar_name: String,
    pub fiscal_year: i32,
    pub current_period: Option<AccountingPeriod>,
    pub open_periods: Vec<AccountingPeriod>,
    pub pending_close_periods: Vec<AccountingPeriod>,
    pub total_checklist_items: i32,
    pub completed_checklist_items: i32,
    pub close_progress_percent: f64,
}

// ============================================================================
// Currency & Exchange Rate Management (Oracle Fusion GL Currency)
// ============================================================================

/// Supported exchange rate types
/// Oracle Fusion: Daily Rates, Spot, Corporate, Period Average, Period End, User
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExchangeRateType {
    #[default]
    Daily,
    Spot,
    Corporate,
    PeriodAverage,
    PeriodEnd,
    User,
    Fixed,
}

/// Currency definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub symbol: String,
    pub precision: i32,
    pub is_base_currency: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update currency request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRequest {
    pub code: String,
    pub name: String,
    pub symbol: Option<String>,
    #[serde(default = "default_precision")]
    pub precision: i32,
    #[serde(default)]
    pub is_base_currency: bool,
}

fn default_precision() -> i32 { 2 }

/// Exchange rate record
/// Oracle Fusion: Daily Rates table with from/to currency and effective date
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub from_currency: String,
    pub to_currency: String,
    pub rate_type: String,
    pub rate: String, // NUMERIC from DB serialized as string
    pub effective_date: chrono::NaiveDate,
    pub inverse_rate: Option<String>,
    pub source: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update exchange rate request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRateRequest {
    pub from_currency: String,
    pub to_currency: String,
    pub rate_type: String,
    pub rate: String,
    pub effective_date: chrono::NaiveDate,
    pub inverse_rate: Option<String>,
    pub source: Option<String>,
}

/// Result of a currency conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyConversionResult {
    pub from_currency: String,
    pub to_currency: String,
    pub from_amount: String,
    pub to_amount: String,
    pub exchange_rate: String,
    pub rate_type: String,
    pub effective_date: chrono::NaiveDate,
    pub gain_loss: Option<String>,
}

/// Unrealized gain/loss on a foreign-currency-denominated balance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnrealizedGainLoss {
    pub currency: String,
    pub original_amount: String,
    pub original_rate: String,
    pub revalued_amount: String,
    pub current_rate: String,
    pub gain_loss_amount: String,
    pub gain_loss_type: String, // "gain" or "loss"
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

// ============================================================================
// Tax Management (Oracle Fusion Tax)
// ============================================================================

/// Tax regime definition
/// Oracle Fusion: Tax Configuration > Tax Regimes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRegime {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub tax_type: String,
    pub default_inclusive: bool,
    pub allows_recovery: bool,
    pub rounding_rule: String,
    pub rounding_precision: i32,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update tax regime request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRegimeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_tax_type")]
    pub tax_type: String,
    #[serde(default)]
    pub default_inclusive: bool,
    #[serde(default)]
    pub allows_recovery: bool,
    #[serde(default = "default_rounding_rule")]
    pub rounding_rule: String,
    #[serde(default = "default_rounding_precision")]
    pub rounding_precision: i32,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_tax_type() -> String { "vat".to_string() }
fn default_rounding_rule() -> String { "nearest".to_string() }
fn default_rounding_precision() -> i32 { 2 }

/// Tax jurisdiction
/// Oracle Fusion: Tax Configuration > Tax Jurisdictions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxJurisdiction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub regime_id: Uuid,
    pub code: String,
    pub name: String,
    pub geographic_level: String,
    pub country_code: Option<String>,
    pub state_code: Option<String>,
    pub county: Option<String>,
    pub city: Option<String>,
    pub postal_code_pattern: Option<String>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update tax jurisdiction request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxJurisdictionRequest {
    pub regime_code: String,
    pub code: String,
    pub name: String,
    #[serde(default = "default_geographic_level")]
    pub geographic_level: String,
    pub country_code: Option<String>,
    pub state_code: Option<String>,
    pub county: Option<String>,
    pub city: Option<String>,
    pub postal_code_pattern: Option<String>,
}

fn default_geographic_level() -> String { "country".to_string() }

/// Tax rate definition
/// Oracle Fusion: Tax Configuration > Tax Rates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub regime_id: Uuid,
    pub jurisdiction_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub rate_percentage: String, // NUMERIC serialized as string
    pub rate_type: String,
    pub tax_account_code: Option<String>,
    pub recoverable: bool,
    pub recovery_percentage: Option<String>, // NUMERIC serialized as string
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update tax rate request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxRateRequest {
    pub regime_code: String,
    pub jurisdiction_code: Option<String>,
    pub code: String,
    pub name: String,
    pub rate_percentage: String,
    #[serde(default = "default_rate_type_tax")]
    pub rate_type: String,
    pub tax_account_code: Option<String>,
    #[serde(default)]
    pub recoverable: bool,
    pub recovery_percentage: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_rate_type_tax() -> String { "standard".to_string() }

/// Tax determination rule
/// Oracle Fusion: Tax Rules > Determination Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxDeterminationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub regime_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub priority: i32,
    pub condition: serde_json::Value,
    pub action: serde_json::Value,
    pub stop_on_match: bool,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Tax line (calculated tax on a transaction)
/// Oracle Fusion: Tax lines attached to invoice/purchase order lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub line_id: Option<Uuid>,
    pub regime_id: Option<Uuid>,
    pub jurisdiction_id: Option<Uuid>,
    pub tax_rate_id: Uuid,
    pub taxable_amount: String,
    pub tax_rate_percentage: String,
    pub tax_amount: String,
    pub is_inclusive: bool,
    pub original_amount: Option<String>,
    pub recoverable_amount: Option<String>,
    pub non_recoverable_amount: Option<String>,
    pub tax_account_code: Option<String>,
    pub determination_rule_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Tax calculation request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxCalculationRequest {
    /// Entity type (e.g., "sales_orders", "purchase_orders")
    pub entity_type: String,
    /// Entity ID
    pub entity_id: Option<Uuid>,
    /// Line items to calculate tax for
    pub lines: Vec<TaxCalculationLine>,
    /// Transaction context for determination rules
    pub context: serde_json::Value,
    /// Whether to persist the tax lines
    #[serde(default)]
    pub persist: bool,
}

/// Single line for tax calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxCalculationLine {
    /// Optional line ID (for linking back)
    pub line_id: Option<Uuid>,
    /// Line amount (net)
    pub amount: String,
    /// Optional product category for determination
    pub product_category: Option<String>,
    /// Optional product code
    pub product_code: Option<String>,
    /// Optional ship-from country
    pub ship_from_country: Option<String>,
    /// Optional ship-to country
    pub ship_to_country: Option<String>,
    /// Optional ship-to state/province
    pub ship_to_state: Option<String>,
    /// Optional specific tax rate codes to apply (bypasses determination)
    pub tax_rate_codes: Option<Vec<String>>,
    /// Whether the amount includes tax already
    pub is_inclusive: Option<bool>,
}

/// Tax calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxCalculationResult {
    pub lines: Vec<TaxLineResult>,
    pub total_taxable_amount: String,
    pub total_tax_amount: String,
    pub total_recoverable_amount: String,
    pub total_non_recoverable_amount: String,
}

/// Tax calculation result for a single line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxLineResult {
    pub line_id: Option<Uuid>,
    pub regime_code: Option<String>,
    pub jurisdiction_code: Option<String>,
    pub tax_rate_code: String,
    pub tax_rate_name: String,
    pub rate_percentage: String,
    pub taxable_amount: String,
    pub tax_amount: String,
    pub is_inclusive: bool,
    pub recoverable: bool,
    pub recovery_percentage: Option<String>,
    pub recoverable_amount: Option<String>,
    pub non_recoverable_amount: Option<String>,
}

// ============================================================================
// Intercompany Transactions (Oracle Fusion Intercompany)
// ============================================================================

/// Intercompany transaction batch
/// Oracle Fusion: Intercompany > Intercompany Batches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub description: Option<String>,
    /// 'draft', 'submitted', 'approved', 'posted', 'cancelled'
    pub status: String,
    pub from_entity_id: Uuid,
    pub from_entity_name: String,
    pub to_entity_id: Uuid,
    pub to_entity_name: String,
    pub currency_code: String,
    pub total_amount: String,
    pub total_debit: String,
    pub total_credit: String,
    pub transaction_count: i32,
    pub from_journal_id: Option<Uuid>,
    pub to_journal_id: Option<Uuid>,
    pub accounting_date: Option<chrono::NaiveDate>,
    pub posted_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create intercompany batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyBatchRequest {
    pub batch_number: String,
    pub description: Option<String>,
    pub from_entity_id: Uuid,
    pub from_entity_name: String,
    pub to_entity_id: Uuid,
    pub to_entity_name: String,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub accounting_date: Option<chrono::NaiveDate>,
}

fn default_currency_usd() -> String { "USD".to_string() }

/// Intercompany transaction (individual line within a batch)
/// Oracle Fusion: Intercompany > Intercompany Transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub transaction_number: String,
    /// 'invoice', 'journal_entry', 'payment', 'charge', 'allocation'
    pub transaction_type: String,
    pub description: Option<String>,
    pub from_entity_id: Uuid,
    pub from_entity_name: String,
    pub to_entity_id: Uuid,
    pub to_entity_name: String,
    pub amount: String,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub from_debit_account: Option<String>,
    pub from_credit_account: Option<String>,
    pub to_debit_account: Option<String>,
    pub to_credit_account: Option<String>,
    pub from_ic_account: String,
    pub to_ic_account: String,
    /// 'draft', 'approved', 'posted', 'settled', 'cancelled'
    pub status: String,
    pub transaction_date: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub settlement_date: Option<chrono::NaiveDate>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create intercompany transaction request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyTransactionRequest {
    pub batch_number: String,
    #[serde(default = "default_ic_transaction_type")]
    pub transaction_type: String,
    pub description: Option<String>,
    pub from_entity_id: Uuid,
    pub from_entity_name: String,
    pub to_entity_id: Uuid,
    pub to_entity_name: String,
    pub amount: String,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub from_debit_account: Option<String>,
    pub from_credit_account: Option<String>,
    pub to_debit_account: Option<String>,
    pub to_credit_account: Option<String>,
    pub from_ic_account: Option<String>,
    pub to_ic_account: Option<String>,
    pub transaction_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<Uuid>,
}

fn default_ic_transaction_type() -> String { "invoice".to_string() }

/// Intercompany settlement
/// Oracle Fusion: Intercompany > Settlements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanySettlement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub settlement_number: String,
    /// 'cash', 'netting', 'offset'
    pub settlement_method: String,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub settled_amount: String,
    pub currency_code: String,
    pub payment_reference: Option<String>,
    /// 'pending', 'completed', 'cancelled'
    pub status: String,
    pub settlement_date: chrono::NaiveDate,
    pub transaction_ids: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create intercompany settlement request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanySettlementRequest {
    pub settlement_number: String,
    #[serde(default = "default_settlement_method")]
    pub settlement_method: String,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub settled_amount: String,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub payment_reference: Option<String>,
    pub transaction_ids: Option<Vec<Uuid>>,
}

fn default_settlement_method() -> String { "cash".to_string() }

/// Intercompany balance (outstanding due-to/due-from between entities)
/// Oracle Fusion: Intercompany > Balances Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyBalance {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub from_entity_id: Uuid,
    pub to_entity_id: Uuid,
    pub currency_code: String,
    pub total_outstanding: String,
    pub total_posted: String,
    pub total_settled: String,
    pub open_transaction_count: i32,
    pub as_of_date: chrono::NaiveDate,
    pub metadata: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// Intercompany balance summary across all entity pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntercompanyBalanceSummary {
    pub total_outstanding: String,
    pub entity_pairs: i32,
    pub open_transactions: i32,
    pub balances: Vec<IntercompanyBalance>,
}

/// Tax report summary
/// Oracle Fusion: Tax Reporting > Tax Filing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub regime_id: Uuid,
    pub jurisdiction_id: Option<Uuid>,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub total_taxable_amount: String,
    pub total_tax_amount: String,
    pub total_recoverable_amount: String,
    pub total_non_recoverable_amount: String,
    pub transaction_count: i32,
    pub status: String,
    pub filed_by: Option<Uuid>,
    pub filed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Bank Reconciliation (Oracle Fusion Cash Management)
// ============================================================================

/// Bank account definition
/// Oracle Fusion: Cash Management > Bank Accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccount {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub account_number: String,
    pub account_name: String,
    pub bank_name: String,
    pub bank_code: Option<String>,
    pub branch_name: Option<String>,
    pub branch_code: Option<String>,
    pub gl_account_code: Option<String>,
    pub currency_code: String,
    pub account_type: String,
    pub last_statement_balance: serde_json::Value,
    pub last_statement_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Create/update bank account request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankAccountRequest {
    pub account_number: String,
    pub account_name: String,
    pub bank_name: String,
    pub bank_code: Option<String>,
    pub branch_name: Option<String>,
    pub branch_code: Option<String>,
    pub gl_account_code: Option<String>,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    #[serde(default = "default_checking")]
    pub account_type: String,
}

fn default_checking() -> String { "checking".to_string() }

/// Bank statement header
/// Oracle Fusion: Cash Management > Bank Statements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankStatement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub bank_account_id: Uuid,
    pub statement_number: String,
    pub statement_date: chrono::NaiveDate,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub opening_balance: serde_json::Value,
    pub closing_balance: serde_json::Value,
    pub total_deposits: serde_json::Value,
    pub total_withdrawals: serde_json::Value,
    pub total_interest: serde_json::Value,
    pub total_charges: serde_json::Value,
    pub total_lines: i32,
    pub matched_lines: i32,
    pub unmatched_lines: i32,
    pub status: String,
    pub reconciliation_percent: serde_json::Value,
    pub imported_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub reconciled_by: Option<Uuid>,
    pub reconciled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Bank statement line
/// Oracle Fusion: Individual line items within a bank statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankStatementLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub statement_id: Uuid,
    pub line_number: i32,
    pub transaction_date: chrono::NaiveDate,
    pub transaction_type: String,
    pub amount: serde_json::Value,
    pub description: Option<String>,
    pub reference_number: Option<String>,
    pub check_number: Option<String>,
    pub counterparty_name: Option<String>,
    pub counterparty_account: Option<String>,
    pub match_status: String,
    pub matched_by: Option<Uuid>,
    pub matched_at: Option<DateTime<Utc>>,
    pub match_method: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// System transaction (AP payment, AR receipt, GL entry)
/// Oracle Fusion: Reconciliation sources
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub bank_account_id: Uuid,
    pub source_type: String,
    pub source_id: Uuid,
    pub source_number: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub amount: serde_json::Value,
    pub transaction_type: String,
    pub description: Option<String>,
    pub reference_number: Option<String>,
    pub check_number: Option<String>,
    pub counterparty_name: Option<String>,
    pub status: String,
    pub gl_posting_date: Option<chrono::NaiveDate>,
    pub currency_code: String,
    pub exchange_rate: Option<serde_json::Value>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Reconciliation match record
/// Oracle Fusion: Links between statement lines and system transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationMatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub statement_id: Uuid,
    pub statement_line_id: Uuid,
    pub system_transaction_id: Uuid,
    pub match_method: String,
    pub match_confidence: Option<serde_json::Value>,
    pub matched_by: Option<Uuid>,
    pub matched_at: Option<DateTime<Utc>>,
    pub unmatched_by: Option<Uuid>,
    pub unmatched_at: Option<DateTime<Utc>>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Reconciliation summary (per account per period)
/// Oracle Fusion: Reconciliation Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationSummary {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub bank_account_id: Uuid,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub statement_id: Option<Uuid>,
    pub statement_balance: serde_json::Value,
    pub book_balance: serde_json::Value,
    pub deposits_in_transit: serde_json::Value,
    pub outstanding_checks: serde_json::Value,
    pub bank_charges: serde_json::Value,
    pub bank_interest: serde_json::Value,
    pub errors_and_omissions: serde_json::Value,
    pub adjusted_book_balance: serde_json::Value,
    pub adjusted_bank_balance: serde_json::Value,
    pub difference: serde_json::Value,
    pub is_balanced: bool,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Auto-matching rule
/// Oracle Fusion: User-defined reconciliation matching rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationMatchingRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub bank_account_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub priority: i32,
    pub criteria: serde_json::Value,
    pub stop_on_match: bool,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create matching rule request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchingRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub bank_account_id: Option<Uuid>,
    #[serde(default = "default_priority")]
    pub priority: i32,
    pub criteria: serde_json::Value,
    #[serde(default = "default_true_val")]
    pub stop_on_match: bool,
}

fn default_priority() -> i32 { 100 }
fn default_true_val() -> bool { true }

/// Auto-match result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoMatchResult {
    pub total_lines: i32,
    pub matched: i32,
    pub unmatched: i32,
    pub already_matched: i32,
    pub matches: Vec<AutoMatchPair>,
}

/// A single auto-matched pair
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoMatchPair {
    pub statement_line_id: Uuid,
    pub system_transaction_id: Uuid,
    pub match_method: String,
    pub confidence: f64,
}

// ============================================================================
// Expense Management (Oracle Fusion Expenses)
// ============================================================================

/// Expense category definition
/// Oracle Fusion: Expenses > Expense Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Whether this category requires a receipt above a threshold
    pub receipt_required: bool,
    /// Amount threshold above which a receipt is required
    pub receipt_threshold: Option<String>,
    /// Whether this category is eligible for per-diem
    pub is_per_diem: bool,
    /// Default per-diem rate (if is_per_diem)
    pub default_per_diem_rate: Option<String>,
    /// Whether this category is eligible for mileage
    pub is_mileage: bool,
    /// Default mileage rate per unit (if is_mileage)
    pub default_mileage_rate: Option<String>,
    /// GL account code for posting
    pub expense_account_code: Option<String>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Expense policy definition
/// Oracle Fusion: Expenses > Expense Policies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpensePolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// The expense category this policy applies to (None = all categories)
    pub category_id: Option<Uuid>,
    /// Minimum expense amount that triggers policy
    pub min_amount: Option<String>,
    /// Maximum expense amount allowed without special approval
    pub max_amount: Option<String>,
    /// Maximum daily total for the category
    pub daily_limit: Option<String>,
    /// Maximum total per expense report for the category
    pub report_limit: Option<String>,
    /// Whether violations require manager approval
    pub requires_approval_on_violation: bool,
    /// Action on violation: "warn", "block", "require_justification"
    pub violation_action: String,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Expense report header
/// Oracle Fusion: Expenses > Expense Reports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub report_number: String,
    pub title: String,
    pub description: Option<String>,
    /// 'draft', 'submitted', 'approved', 'rejected', 'reimbursed', 'cancelled'
    pub status: String,
    /// Employee who submitted the report
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    /// Department for cost center
    pub department_id: Option<Uuid>,
    /// Purpose of the expense
    pub purpose: Option<String>,
    /// Project reference (for project billing)
    pub project_id: Option<Uuid>,
    /// Currency code
    pub currency_code: String,
    /// Total amount of all expense lines
    pub total_amount: String,
    /// Total reimbursable amount
    pub reimbursable_amount: String,
    /// Total amount requiring receipts
    pub receipt_required_amount: String,
    /// Number of attached receipts
    pub receipt_count: i32,
    /// Business trip start date
    pub trip_start_date: Option<chrono::NaiveDate>,
    /// Business trip end date
    pub trip_end_date: Option<chrono::NaiveDate>,
    /// Cost center override
    pub cost_center: Option<String>,
    /// Approval information
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    /// Payment information
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub reimbursed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual expense line within a report
/// Oracle Fusion: Expense Lines within Expense Reports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub report_id: Uuid,
    pub line_number: i32,
    pub expense_category_id: Option<Uuid>,
    pub expense_category_name: Option<String>,
    /// 'expense', 'per_diem', 'mileage', 'credit_card'
    pub expense_type: String,
    /// Free-text description of the expense
    pub description: Option<String>,
    /// Date the expense was incurred
    pub expense_date: chrono::NaiveDate,
    /// Amount in the report currency
    pub amount: String,
    /// Original currency if different from report currency
    pub original_currency: Option<String>,
    /// Original amount in the foreign currency
    pub original_amount: Option<String>,
    /// Exchange rate applied
    pub exchange_rate: Option<String>,
    /// Whether this expense is reimbursable
    pub is_reimbursable: bool,
    /// Whether a receipt is attached
    pub has_receipt: bool,
    /// Receipt attachment reference
    pub receipt_reference: Option<String>,
    /// Merchant / vendor name
    pub merchant_name: Option<String>,
    /// Location where expense was incurred
    pub location: Option<String>,
    /// Attendees (for entertainment / meals)
    pub attendees: Option<serde_json::Value>,
    /// For per-diem: number of days
    pub per_diem_days: Option<f64>,
    /// For per-diem: daily rate
    pub per_diem_rate: Option<String>,
    /// For mileage: distance
    pub mileage_distance: Option<f64>,
    /// For mileage: rate per unit
    pub mileage_rate: Option<String>,
    /// For mileage: unit ("miles" or "km")
    pub mileage_unit: Option<String>,
    /// For mileage: starting location
    pub mileage_from: Option<String>,
    /// For mileage: ending location
    pub mileage_to: Option<String>,
    /// Whether this line violates any expense policy
    pub policy_violation: bool,
    /// Policy violation details
    pub policy_violation_message: Option<String>,
    /// GL account code override
    pub expense_account_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Expense policy violation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpensePolicyViolation {
    pub line_id: Uuid,
    pub policy_id: Uuid,
    pub policy_name: String,
    pub field: String,
    pub message: String,
    pub severity: String, // "warning" or "error"
}

// ============================================================================
// Budget Management (Oracle Fusion General Ledger > Budgets)
// ============================================================================

/// Budget definition (template)
/// Oracle Fusion: General Ledger > Budgets > Define Budget
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Budget code (e.g., 'FY2024_OPEx', 'FY2024_CAPEx')
    pub code: String,
    /// Display name
    pub name: String,
    pub description: Option<String>,
    /// Reference to accounting calendar
    pub calendar_id: Option<Uuid>,
    /// Fiscal year this budget covers
    pub fiscal_year: Option<i32>,
    /// Budget type: 'operating', 'capital', 'project', 'cash_flow'
    pub budget_type: String,
    /// Control level: 'none', 'advisory', 'absolute'
    pub control_level: String,
    /// Whether carry-forward of unspent amounts is allowed
    pub allow_carry_forward: bool,
    /// Whether transfers between accounts are allowed
    pub allow_transfers: bool,
    /// Default currency
    pub currency_code: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update budget definition request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetDefinitionRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub calendar_id: Option<Uuid>,
    pub fiscal_year: Option<i32>,
    #[serde(default = "default_budget_type")]
    pub budget_type: String,
    #[serde(default = "default_control_level")]
    pub control_level: String,
    #[serde(default)]
    pub allow_carry_forward: bool,
    #[serde(default = "default_true")]
    pub allow_transfers: bool,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
}

fn default_budget_type() -> String { "operating".to_string() }
fn default_control_level() -> String { "none".to_string() }

/// Budget version (snapshot with workflow)
/// Oracle Fusion: General Ledger > Budgets > Budget Versions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetVersion {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// The budget definition this version belongs to
    pub definition_id: Uuid,
    /// Auto-incremented version number
    pub version_number: i32,
    /// Version label (e.g., 'Original', 'Revised Q2')
    pub label: Option<String>,
    /// Status: 'draft', 'submitted', 'approved', 'active', 'closed', 'rejected'
    pub status: String,
    /// Totals (calculated from budget lines)
    pub total_budget_amount: String,
    pub total_committed_amount: String,
    pub total_actual_amount: String,
    pub total_variance_amount: String,
    /// Approval workflow
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    /// Effective dates
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    /// Notes
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create budget version request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetVersionRequest {
    /// Budget definition code to create version for
    pub budget_code: String,
    pub label: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
}

/// Budget line (individual budget amount by account/period/dimension)
/// Oracle Fusion: General Ledger > Budgets > Enter Budget Amounts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Budget version reference
    pub version_id: Uuid,
    /// Line number
    pub line_number: i32,
    /// Account reference
    pub account_code: String,
    pub account_name: Option<String>,
    /// Period reference
    pub period_name: Option<String>,
    pub period_start_date: Option<chrono::NaiveDate>,
    pub period_end_date: Option<chrono::NaiveDate>,
    pub fiscal_year: Option<i32>,
    pub quarter: Option<i32>,
    /// Dimension references
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub cost_center: Option<String>,
    /// Budget amounts
    pub budget_amount: String,
    pub committed_amount: String,
    pub actual_amount: String,
    pub variance_amount: String,
    pub variance_percent: String,
    /// Carry-forward
    pub carry_forward_amount: String,
    /// Transfer tracking
    pub transferred_in_amount: String,
    pub transferred_out_amount: String,
    /// Description
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update budget line request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetLineRequest {
    pub account_code: String,
    pub account_name: Option<String>,
    pub period_name: Option<String>,
    pub period_start_date: Option<chrono::NaiveDate>,
    pub period_end_date: Option<chrono::NaiveDate>,
    pub fiscal_year: Option<i32>,
    pub quarter: Option<i32>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub cost_center: Option<String>,
    /// The budgeted amount
    pub budget_amount: String,
    pub description: Option<String>,
}

/// Budget transfer
/// Oracle Fusion: General Ledger > Budgets > Transfer Budget Amounts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetTransfer {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Budget version reference
    pub version_id: Uuid,
    /// Transfer number
    pub transfer_number: String,
    pub description: Option<String>,
    /// Source account
    pub from_account_code: String,
    pub from_period_name: Option<String>,
    pub from_department_id: Option<Uuid>,
    pub from_cost_center: Option<String>,
    /// Destination account
    pub to_account_code: String,
    pub to_period_name: Option<String>,
    pub to_department_id: Option<Uuid>,
    pub to_cost_center: Option<String>,
    /// Amount to transfer
    pub amount: String,
    /// Status: 'pending', 'approved', 'rejected', 'cancelled'
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create budget transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetTransferRequest {
    pub budget_code: String,
    pub from_account_code: String,
    pub from_period_name: Option<String>,
    pub from_department_id: Option<Uuid>,
    pub from_cost_center: Option<String>,
    pub to_account_code: String,
    pub to_period_name: Option<String>,
    pub to_department_id: Option<Uuid>,
    pub to_cost_center: Option<String>,
    pub amount: String,
    pub description: Option<String>,
}

/// Budget vs Actuals variance report
/// Oracle Fusion: General Ledger > Budgets > Budget vs Actuals Report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetVarianceReport {
    pub definition_id: Uuid,
    pub definition_code: String,
    pub definition_name: String,
    pub version_id: Uuid,
    pub version_label: Option<String>,
    pub fiscal_year: Option<i32>,
    /// Summary totals
    pub total_budget: String,
    pub total_actual: String,
    pub total_committed: String,
    pub total_variance: String,
    pub variance_percent: String,
    /// Line-by-line details
    pub lines: Vec<BudgetVarianceLine>,
}

/// Single line in the budget variance report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetVarianceLine {
    pub account_code: String,
    pub account_name: Option<String>,
    pub period_name: Option<String>,
    pub department_name: Option<String>,
    pub project_name: Option<String>,
    pub cost_center: Option<String>,
    pub budget_amount: String,
    pub committed_amount: String,
    pub actual_amount: String,
    pub variance_amount: String,
    pub variance_percent: String,
    /// Whether this line is over budget
    pub is_over_budget: bool,
}

// ============================================================================
// Fixed Assets Management (Oracle Fusion Fixed Assets)
// ============================================================================

/// Asset category definition
/// Oracle Fusion: Fixed Assets > Asset Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Default depreciation method for assets in this category
    /// 'straight_line', 'declining_balance', 'sum_of_years_digits'
    pub default_depreciation_method: String,
    /// Default useful life in months
    pub default_useful_life_months: i32,
    /// Default salvage value percentage
    pub default_salvage_value_percent: String,
    /// Default GL account codes
    pub default_asset_account_code: Option<String>,
    pub default_accum_depr_account_code: Option<String>,
    pub default_depr_expense_account_code: Option<String>,
    pub default_gain_loss_account_code: Option<String>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset book definition (corporate, tax)
/// Oracle Fusion: Fixed Assets > Books
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetBook {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Book type: 'corporate', 'tax'
    pub book_type: String,
    pub auto_depreciation: bool,
    /// Depreciation calendar: 'monthly', 'quarterly', 'yearly'
    pub depreciation_calendar: String,
    pub current_fiscal_year: Option<i32>,
    pub last_depreciation_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fixed asset record
/// Oracle Fusion: Fixed Assets > Assets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixedAsset {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub asset_number: String,
    pub asset_name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_code: Option<String>,
    pub book_id: Option<Uuid>,
    pub book_code: Option<String>,
    /// Asset type: 'tangible', 'intangible', 'leased', 'cipc'
    pub asset_type: String,
    /// Lifecycle status: 'draft', 'acquired', 'in_service', 'under_construction',
    /// 'disposed', 'retired', 'transferred'
    pub status: String,
    // Financial details
    pub original_cost: String,
    pub current_cost: String,
    pub salvage_value: String,
    pub salvage_value_percent: String,
    // Depreciation parameters
    pub depreciation_method: String,
    pub useful_life_months: i32,
    pub declining_balance_rate: Option<String>,
    // Depreciation calculations
    pub depreciable_basis: String,
    pub accumulated_depreciation: String,
    pub net_book_value: String,
    pub depreciation_per_period: String,
    // Depreciation tracking
    pub periods_depreciated: i32,
    pub last_depreciation_date: Option<chrono::NaiveDate>,
    pub last_depreciation_amount: String,
    // Date tracking
    pub acquisition_date: Option<chrono::NaiveDate>,
    pub in_service_date: Option<chrono::NaiveDate>,
    pub disposal_date: Option<chrono::NaiveDate>,
    pub retirement_date: Option<chrono::NaiveDate>,
    // Location and assignment
    pub location: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub custodian_id: Option<Uuid>,
    pub custodian_name: Option<String>,
    // Physical details
    pub serial_number: Option<String>,
    pub tag_number: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    // Dates
    pub warranty_expiry: Option<chrono::NaiveDate>,
    pub insurance_policy_number: Option<String>,
    pub insurance_expiry: Option<chrono::NaiveDate>,
    pub lease_number: Option<String>,
    pub lease_expiry: Option<chrono::NaiveDate>,
    // GL account codes
    pub asset_account_code: Option<String>,
    pub accum_depr_account_code: Option<String>,
    pub depr_expense_account_code: Option<String>,
    pub gain_loss_account_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset depreciation history entry
/// Oracle Fusion: Fixed Assets > Depreciation History
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDepreciationHistory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub asset_id: Uuid,
    pub fiscal_year: i32,
    pub period_number: i32,
    pub period_name: Option<String>,
    pub depreciation_date: chrono::NaiveDate,
    pub depreciation_amount: String,
    pub accumulated_depreciation: String,
    pub net_book_value: String,
    pub depreciation_method: String,
    pub journal_entry_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Asset transfer record
/// Oracle Fusion: Fixed Assets > Asset Transfers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTransfer {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transfer_number: String,
    pub asset_id: Uuid,
    // From
    pub from_department_id: Option<Uuid>,
    pub from_department_name: Option<String>,
    pub from_location: Option<String>,
    pub from_custodian_id: Option<Uuid>,
    pub from_custodian_name: Option<String>,
    // To
    pub to_department_id: Option<Uuid>,
    pub to_department_name: Option<String>,
    pub to_location: Option<String>,
    pub to_custodian_id: Option<Uuid>,
    pub to_custodian_name: Option<String>,
    // Details
    pub transfer_date: chrono::NaiveDate,
    pub reason: Option<String>,
    /// Status: 'pending', 'approved', 'rejected', 'completed'
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset retirement record
/// Oracle Fusion: Fixed Assets > Asset Retirements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetRetirement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub retirement_number: String,
    pub asset_id: Uuid,
    /// Retirement type: 'sale', 'scrap', 'donation', 'write_off', 'casualty'
    pub retirement_type: String,
    pub retirement_date: chrono::NaiveDate,
    // Financial details
    pub proceeds: String,
    pub removal_cost: String,
    pub net_book_value_at_retirement: String,
    pub accumulated_depreciation_at_retirement: String,
    pub gain_loss_amount: String,
    /// 'gain' or 'loss'
    pub gain_loss_type: Option<String>,
    // Account references
    pub gain_account_code: Option<String>,
    pub loss_account_code: Option<String>,
    pub cash_account_code: Option<String>,
    pub asset_account_code: Option<String>,
    pub accum_depr_account_code: Option<String>,
    // Reference
    pub reference_number: Option<String>,
    pub buyer_name: Option<String>,
    pub notes: Option<String>,
    /// Status: 'pending', 'approved', 'completed', 'cancelled'
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub journal_entry_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Collections & Credit Management (Oracle Fusion Collections)
// ============================================================================

/// Customer credit profile
/// Oracle Fusion: Collections > Customer Credit Profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerCreditProfile {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    /// Credit limit amount
    pub credit_limit: String,
    /// Currently used credit
    pub credit_used: String,
    /// Available credit (limit - used)
    pub credit_available: String,
    /// Risk classification: 'low', 'medium', 'high', 'very_high', 'defaulted'
    pub risk_classification: String,
    /// Internal credit score (0-1000)
    pub credit_score: Option<i32>,
    /// External credit rating
    pub external_credit_rating: Option<String>,
    pub external_rating_agency: Option<String>,
    pub external_rating_date: Option<chrono::NaiveDate>,
    /// Default payment terms
    pub payment_terms: String,
    /// Average days to pay
    pub average_days_to_pay: Option<String>,
    /// Overdue invoice count
    pub overdue_invoice_count: i32,
    /// Total overdue amount
    pub total_overdue_amount: String,
    /// Oldest overdue date
    pub oldest_overdue_date: Option<chrono::NaiveDate>,
    /// Whether customer is on credit hold
    pub credit_hold: bool,
    pub credit_hold_reason: Option<String>,
    pub credit_hold_date: Option<DateTime<Utc>>,
    pub credit_hold_by: Option<Uuid>,
    /// Review dates
    pub last_review_date: Option<chrono::NaiveDate>,
    pub next_review_date: Option<chrono::NaiveDate>,
    /// 'active', 'inactive', 'blocked'
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update credit profile request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditProfileRequest {
    pub customer_id: Uuid,
    pub credit_limit: String,
    #[serde(default = "default_risk_medium")]
    pub risk_classification: String,
    pub credit_score: Option<i32>,
    pub external_credit_rating: Option<String>,
    pub external_rating_agency: Option<String>,
    pub external_rating_date: Option<chrono::NaiveDate>,
    #[serde(default = "default_net_30")]
    pub payment_terms: String,
    pub next_review_date: Option<chrono::NaiveDate>,
}

fn default_risk_medium() -> String { "medium".to_string() }
fn default_net_30() -> String { "net_30".to_string() }

/// Collection strategy definition
/// Oracle Fusion: Collections > Collection Strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionStrategy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// 'automatic' or 'manual'
    pub strategy_type: String,
    /// Applicable risk classifications
    pub applicable_risk_classifications: serde_json::Value,
    /// Aging buckets that trigger this strategy
    pub trigger_aging_buckets: serde_json::Value,
    /// Overdue amount threshold
    pub overdue_amount_threshold: String,
    /// Ordered collection actions
    pub actions: serde_json::Value,
    /// Priority
    pub priority: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Collection case
/// Oracle Fusion: Collections > Collection Cases
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionCase {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub case_number: String,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub strategy_id: Option<Uuid>,
    /// Assigned collector
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    /// 'collection', 'dispute', 'bankruptcy', 'skip_trace'
    pub case_type: String,
    /// 'open', 'in_progress', 'resolved', 'closed', 'escalated', 'written_off'
    pub status: String,
    /// 'low', 'medium', 'high', 'critical'
    pub priority: String,
    /// Financial summary
    pub total_overdue_amount: String,
    pub total_disputed_amount: String,
    pub total_invoiced_amount: String,
    pub overdue_invoice_count: i32,
    pub oldest_overdue_date: Option<chrono::NaiveDate>,
    /// Current strategy step
    pub current_step: i32,
    /// Key dates
    pub opened_date: chrono::NaiveDate,
    pub target_resolution_date: Option<chrono::NaiveDate>,
    pub resolved_date: Option<chrono::NaiveDate>,
    pub closed_date: Option<chrono::NaiveDate>,
    pub last_action_date: Option<chrono::NaiveDate>,
    pub next_action_date: Option<chrono::NaiveDate>,
    /// Resolution
    pub resolution_type: Option<String>,
    pub resolution_notes: Option<String>,
    /// Related invoices
    pub related_invoice_ids: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Customer interaction record
/// Oracle Fusion: Collections > Customer Interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerInteraction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub case_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    /// 'phone_call', 'email', 'letter', 'meeting', 'note', 'sms'
    pub interaction_type: String,
    /// 'outbound', 'inbound'
    pub direction: String,
    pub contact_name: Option<String>,
    pub contact_role: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    /// Outcome: 'contacted', 'left_message', 'no_answer', 'promised_to_pay',
    /// 'disputed', 'refused', 'agreed_payment_plan', 'escalated', 'no_action'
    pub outcome: Option<String>,
    pub follow_up_date: Option<chrono::NaiveDate>,
    pub follow_up_notes: Option<String>,
    pub performed_by: Option<Uuid>,
    pub performed_by_name: Option<String>,
    pub performed_at: Option<DateTime<Utc>>,
    pub duration_minutes: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Promise to pay
/// Oracle Fusion: Collections > Promises to Pay
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromiseToPay {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub case_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    /// 'single_payment', 'installment', 'full_balance'
    pub promise_type: String,
    pub promised_amount: String,
    pub paid_amount: String,
    pub remaining_amount: String,
    pub promise_date: chrono::NaiveDate,
    pub installment_count: Option<i32>,
    pub installment_frequency: Option<String>,
    /// 'pending', 'partially_kept', 'kept', 'broken', 'cancelled'
    pub status: String,
    pub broken_date: Option<chrono::NaiveDate>,
    pub broken_reason: Option<String>,
    pub related_invoice_ids: serde_json::Value,
    pub promised_by_name: Option<String>,
    pub promised_by_role: Option<String>,
    pub notes: Option<String>,
    pub recorded_by: Option<Uuid>,
    pub recorded_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dunning campaign
/// Oracle Fusion: Collections > Dunning Management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DunningCampaign {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub campaign_number: String,
    pub name: String,
    pub description: Option<String>,
    /// 'reminder', 'first_notice', 'second_notice', 'final_notice', 'pre_legal', 'legal'
    pub dunning_level: String,
    /// 'email', 'letter', 'sms', 'phone'
    pub communication_method: String,
    pub template_id: Option<Uuid>,
    pub template_name: Option<String>,
    pub min_overdue_days: i32,
    pub min_overdue_amount: String,
    pub target_risk_classifications: serde_json::Value,
    pub exclude_active_cases: bool,
    pub scheduled_date: Option<chrono::NaiveDate>,
    pub sent_date: Option<chrono::NaiveDate>,
    pub target_customer_count: i32,
    pub sent_count: i32,
    pub failed_count: i32,
    /// 'draft', 'scheduled', 'in_progress', 'completed', 'cancelled'
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dunning letter (individual)
/// Oracle Fusion: Collections > Dunning Letters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DunningLetter {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub campaign_id: Option<Uuid>,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub customer_address: Option<serde_json::Value>,
    pub customer_email: Option<String>,
    pub dunning_level: String,
    pub communication_method: String,
    pub total_overdue_amount: String,
    pub overdue_invoice_count: i32,
    pub oldest_overdue_date: Option<chrono::NaiveDate>,
    /// Aging breakdown
    pub aging_current: String,
    pub aging_1_30: String,
    pub aging_31_60: String,
    pub aging_61_90: String,
    pub aging_91_120: String,
    pub aging_121_plus: String,
    /// 'pending', 'sent', 'delivered', 'bounced', 'failed', 'viewed'
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub viewed_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    pub invoice_details: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receivables aging snapshot
/// Oracle Fusion: Collections > Aging Analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivablesAgingSnapshot {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub snapshot_date: chrono::NaiveDate,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub total_outstanding: String,
    /// Aging buckets
    pub aging_current: String,
    pub aging_1_30: String,
    pub aging_31_60: String,
    pub aging_61_90: String,
    pub aging_91_120: String,
    pub aging_121_plus: String,
    /// Counts per bucket
    pub count_current: i32,
    pub count_1_30: i32,
    pub count_31_60: i32,
    pub count_61_90: i32,
    pub count_91_120: i32,
    pub count_121_plus: i32,
    pub weighted_average_days_overdue: Option<String>,
    pub overdue_percent: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Write-off request
/// Oracle Fusion: Collections > Write-Off Management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteOffRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_number: String,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    /// 'bad_debt', 'small_balance', 'dispute', 'adjustment'
    pub write_off_type: String,
    pub write_off_amount: String,
    pub write_off_account_code: Option<String>,
    pub reason: String,
    pub related_invoice_ids: serde_json::Value,
    pub case_id: Option<Uuid>,
    /// 'draft', 'submitted', 'approved', 'rejected', 'processed', 'cancelled'
    pub status: String,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Aging summary report
/// Oracle Fusion: Collections > Aging Report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgingSummary {
    pub organization_id: Uuid,
    pub as_of_date: chrono::NaiveDate,
    pub total_outstanding: String,
    pub total_overdue: String,
    pub aging_current: String,
    pub aging_1_30: String,
    pub aging_31_60: String,
    pub aging_61_90: String,
    pub aging_91_120: String,
    pub aging_121_plus: String,
    pub customer_count: i32,
    pub overdue_customer_count: i32,
    pub weighted_average_days_overdue: String,
}

// ============================================================================
// Revenue Recognition (ASC 606 / IFRS 15)
// Oracle Fusion Cloud ERP: Financials > Revenue Management
// ============================================================================

/// Revenue Recognition Policy
/// Defines the accounting policy for revenue recognition.
/// Oracle Fusion equivalent: Revenue Management > Policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenuePolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique policy code (e.g., "STD_SaaS", "STD_CONSULTING")
    pub code: String,
    /// Human-readable name
    pub name: String,
    /// Description of the policy
    pub description: Option<String>,
    /// Recognition method: "over_time", "point_in_time"
    pub recognition_method: String,
    /// Over-time method (when recognition_method = over_time):
    /// "output", "input", "straight_line"
    pub over_time_method: Option<String>,
    /// Allocation basis: "standalone_selling_price", "residual", "equal"
    pub allocation_basis: String,
    /// Default standalone selling price (used when SSP is not determined per-product)
    pub default_selling_price: Option<String>,
    /// Whether variable consideration is constrained
    pub constrain_variable_consideration: bool,
    /// Constraint threshold percentage (0-100)
    pub constraint_threshold_percent: Option<String>,
    /// Default revenue account code
    pub revenue_account_code: Option<String>,
    /// Default deferred revenue account code
    pub deferred_revenue_account_code: Option<String>,
    /// Default contra-revenue account code (for allowances)
    pub contra_revenue_account_code: Option<String>,
    /// Whether this policy is active
    pub is_active: bool,
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Revenue Contract (Revenue Arrangement)
/// Represents a customer contract with one or more performance obligations.
/// Oracle Fusion equivalent: Revenue Management > Revenue Contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueContract {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated contract number (e.g., "RC-0001")
    pub contract_number: String,
    /// Reference to the source sales order or agreement
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    /// Customer information
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    /// Contract dates
    pub contract_date: Option<chrono::NaiveDate>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    /// Total transaction price (before allocation)
    pub total_transaction_price: String,
    /// Total allocated revenue across all performance obligations
    pub total_allocated_revenue: String,
    /// Total recognized revenue to date
    pub total_recognized_revenue: String,
    /// Total deferred revenue remaining
    pub total_deferred_revenue: String,
    /// Contract status: "draft", "active", "completed", "cancelled", "modified"
    pub status: String,
    /// ASC 606 step completion tracking
    /// Step 1: Identify the contract
    pub step1_contract_identified: bool,
    /// Step 2: Identify performance obligations (POs created)
    pub step2_obligations_identified: bool,
    /// Step 3: Determine transaction price
    pub step3_price_determined: bool,
    /// Step 4: Allocate transaction price
    pub step4_price_allocated: bool,
    /// Step 5: Recognize revenue
    pub step5_recognition_scheduled: bool,
    /// Currency
    pub currency_code: String,
    /// Optional notes
    pub notes: Option<String>,
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance Obligation
/// A distinct good or service promised in a revenue contract.
/// Oracle Fusion equivalent: Revenue Management > Performance Obligations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceObligation {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent revenue contract
    pub contract_id: Uuid,
    /// Line number within the contract
    pub line_number: i32,
    /// Description of the good or service
    pub description: Option<String>,
    /// Product or service reference
    pub product_id: Option<Uuid>,
    pub product_name: Option<String>,
    /// Reference to source line (e.g., sales order line)
    pub source_line_id: Option<Uuid>,
    /// Revenue policy applied to this obligation
    pub revenue_policy_id: Option<Uuid>,
    /// Recognition method for this specific obligation
    /// (overrides policy default if set)
    pub recognition_method: Option<String>,
    /// Over-time method override
    pub over_time_method: Option<String>,
    /// Standalone selling price (SSP)
    pub standalone_selling_price: String,
    /// Allocated transaction price (after SSP allocation)
    pub allocated_transaction_price: String,
    /// Total recognized revenue for this obligation
    pub total_recognized_revenue: String,
    /// Remaining deferred revenue
    pub deferred_revenue: String,
    /// Recognition start date
    pub recognition_start_date: Option<chrono::NaiveDate>,
    /// Recognition end date (for over-time)
    pub recognition_end_date: Option<chrono::NaiveDate>,
    /// Percent complete (for over-time recognition)
    pub percent_complete: Option<String>,
    /// Satisfaction method: "over_time", "point_in_time"
    pub satisfaction_method: String,
    /// Status: "pending", "in_progress", "satisfied", "partially_satisfied", "cancelled"
    pub status: String,
    /// Revenue account overrides
    pub revenue_account_code: Option<String>,
    pub deferred_revenue_account_code: Option<String>,
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Revenue Recognition Schedule Line
/// Individual revenue recognition events for a performance obligation.
/// Oracle Fusion equivalent: Revenue Management > Revenue Schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueScheduleLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent performance obligation
    pub obligation_id: Uuid,
    /// Parent contract (denormalized for querying)
    pub contract_id: Uuid,
    /// Schedule line number
    pub line_number: i32,
    /// Planned recognition date
    pub recognition_date: chrono::NaiveDate,
    /// Amount to recognize
    pub amount: String,
    /// Amount actually recognized
    pub recognized_amount: String,
    /// Status: "planned", "recognized", "reversed", "cancelled"
    pub status: String,
    /// Recognition method used
    pub recognition_method: Option<String>,
    /// Percentage of total for this line
    pub percent_of_total: Option<String>,
    /// Journal entry reference (posted to GL)
    pub journal_entry_id: Option<Uuid>,
    /// When the recognition was actually posted
    pub recognized_at: Option<DateTime<Utc>>,
    /// Reversal reference
    pub reversed_by_id: Option<Uuid>,
    /// Reason for reversal (if reversed)
    pub reversal_reason: Option<String>,
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Payment Management (Oracle Fusion Payables > Payments)
// ============================================================================

/// Payment terms definition
/// Oracle Fusion: Payables > Setup > Payment Terms
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentTerm {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code (e.g., "NET30", "2_10_NET30", "DUE_ON_RECEIPT")
    pub code: String,
    /// Display name (e.g., "Net 30 Days")
    pub name: String,
    pub description: Option<String>,
    /// Number of days from invoice date until payment is due
    pub due_days: i32,
    /// Days within which a discount is available
    pub discount_days: Option<i32>,
    /// Discount percentage for early payment
    pub discount_percentage: Option<String>,
    /// Whether this is an installment payment term
    pub is_installment: bool,
    /// Number of installments
    pub installment_count: Option<i32>,
    /// Installment frequency: 'monthly', 'quarterly', 'weekly'
    pub installment_frequency: Option<String>,
    /// Default payment method
    pub default_payment_method: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update payment terms request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentTermRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_thirty")]
    pub due_days: i32,
    pub discount_days: Option<i32>,
    pub discount_percentage: Option<String>,
    #[serde(default)]
    pub is_installment: bool,
    pub installment_count: Option<i32>,
    pub installment_frequency: Option<String>,
    pub default_payment_method: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_thirty() -> i32 { 30 }

/// Payment batch (payment run)
/// Oracle Fusion: Payables > Payments > Payment Batches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub name: Option<String>,
    pub description: Option<String>,
    /// When payments should be issued
    pub payment_date: chrono::NaiveDate,
    /// Bank account to pay from
    pub bank_account_id: Option<Uuid>,
    /// Payment method: 'check', 'eft', 'wire', 'ach'
    pub payment_method: String,
    pub currency_code: String,
    /// Selection criteria used to select invoices for payment
    pub selection_criteria: serde_json::Value,
    /// Counts and totals
    pub total_invoice_count: i32,
    pub total_payment_count: i32,
    pub total_payment_amount: String,
    pub total_discount_taken: String,
    /// Status: 'draft', 'selected', 'approved', 'formatted', 'confirmed', 'cancelled'
    pub status: String,
    /// Workflow tracking
    pub selected_by: Option<Uuid>,
    pub selected_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub formatted_by: Option<Uuid>,
    pub formatted_at: Option<DateTime<Utc>>,
    pub confirmed_by: Option<Uuid>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    /// Generated payment file reference
    pub payment_file_name: Option<String>,
    pub payment_file_reference: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create payment batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentBatchRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub payment_date: chrono::NaiveDate,
    pub bank_account_id: Option<Uuid>,
    #[serde(default = "default_check_method")]
    pub payment_method: String,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub selection_criteria: Option<serde_json::Value>,
}

fn default_check_method() -> String { "check".to_string() }

/// Individual payment
/// Oracle Fusion: Payables > Payments > Payments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payment_number: String,
    pub batch_id: Option<Uuid>,
    /// Supplier information
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    pub supplier_site: Option<String>,
    /// Payment details
    pub payment_date: chrono::NaiveDate,
    pub payment_method: String,
    pub currency_code: String,
    /// Amounts
    pub payment_amount: String,
    pub discount_taken: String,
    pub bank_charges: String,
    /// Bank account (source of funds)
    pub bank_account_id: Option<Uuid>,
    pub bank_account_name: Option<String>,
    /// GL account codes
    pub cash_account_code: Option<String>,
    pub ap_account_code: Option<String>,
    pub discount_account_code: Option<String>,
    /// Status: 'draft', 'issued', 'cleared', 'voided', 'reconciled', 'stopped'
    pub status: String,
    /// Check / reference number
    pub check_number: Option<String>,
    pub reference_number: Option<String>,
    /// Void tracking
    pub voided_by: Option<Uuid>,
    pub voided_at: Option<DateTime<Utc>>,
    pub void_reason: Option<String>,
    /// Reissue tracking
    pub reissued_from_payment_id: Option<Uuid>,
    pub reissued_payment_id: Option<Uuid>,
    /// Clearance tracking
    pub cleared_date: Option<chrono::NaiveDate>,
    pub cleared_by: Option<Uuid>,
    pub cleared_at: Option<DateTime<Utc>>,
    /// GL posting
    pub journal_entry_id: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    /// Remittance
    pub remittance_sent: bool,
    pub remittance_sent_at: Option<DateTime<Utc>>,
    pub remittance_method: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment line (invoice covered by a payment)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payment_id: Uuid,
    pub line_number: i32,
    /// Invoice reference
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub invoice_due_date: Option<chrono::NaiveDate>,
    /// Amounts
    pub invoice_amount: Option<String>,
    pub amount_paid: String,
    pub discount_taken: String,
    pub withholding_amount: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Scheduled payment
/// Oracle Fusion: Payables > Payments > Scheduled Payments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledPayment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    /// Scheduling
    pub scheduled_payment_date: chrono::NaiveDate,
    pub scheduled_amount: String,
    pub installment_number: i32,
    pub payment_method: Option<String>,
    pub bank_account_id: Option<Uuid>,
    /// Batch selection
    pub is_selected: bool,
    pub selected_batch_id: Option<Uuid>,
    pub payment_id: Option<Uuid>,
    /// Status: 'pending', 'selected', 'paid', 'cancelled'
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment format
/// Oracle Fusion: Payables > Setup > Payment Formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentFormat {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Format type: 'file', 'printed_check', 'edi', 'xml', 'json'
    pub format_type: String,
    pub template_reference: Option<String>,
    pub applicable_methods: serde_json::Value,
    pub is_system: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Remittance advice
/// Oracle Fusion: Payables > Payments > Remittance Advice
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemittanceAdvice {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payment_id: Uuid,
    /// Delivery
    pub delivery_method: String,
    pub delivery_address: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    /// Content
    pub subject: Option<String>,
    pub body: Option<String>,
    /// Status: 'pending', 'sent', 'delivered', 'failed'
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
    /// Payment summary
    pub payment_summary: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentDashboardSummary {
    pub total_pending_payment_count: i32,
    pub total_pending_payment_amount: String,
    pub total_paid_payment_count: i32,
    pub total_paid_payment_amount: String,
    pub total_discount_taken: String,
    pub payments_by_method: serde_json::Value,
    pub payments_by_status: serde_json::Value,
    /// Upcoming scheduled payments (next 7 days)
    pub upcoming_scheduled_count: i32,
    pub upcoming_scheduled_amount: String,
}

/// Revenue Contract Modification
/// Tracks changes/amendments to revenue contracts.
/// Oracle Fusion equivalent: Revenue Management > Contract Modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueModification {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Contract being modified
    pub contract_id: Uuid,
    /// Modification number (sequential)
    pub modification_number: i32,
    /// Type of modification: "price_change", "scope_change", "term_extension",
    /// "termination", "add_obligation", "remove_obligation"
    pub modification_type: String,
    /// Description of the change
    pub description: Option<String>,
    /// Previous total transaction price
    pub previous_transaction_price: String,
    /// New total transaction price
    pub new_transaction_price: String,
    /// Previous contract end date
    pub previous_end_date: Option<chrono::NaiveDate>,
    /// New contract end date
    pub new_end_date: Option<chrono::NaiveDate>,
    /// Effective date of the modification
    pub effective_date: chrono::NaiveDate,
    /// Status: "draft", "active", "cancelled"
    pub status: String,
    /// Arbitrary metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Subledger Accounting Types
// Oracle Fusion: Financials > General Ledger > Subledger Accounting
// ============================================================================

/// Accounting Method
/// Defines how a subledger transaction type is accounted for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingMethod {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Application: 'payables', 'receivables', 'expenses', 'assets', 'projects'
    pub application: String,
    /// Transaction type within the application
    pub transaction_type: String,
    /// Event class: 'create', 'update', 'cancel', 'reverse'
    pub event_class: String,
    pub auto_accounting: bool,
    pub allow_manual_entries: bool,
    pub apply_rounding: bool,
    pub rounding_account_code: Option<String>,
    pub rounding_threshold: String,
    pub require_balancing: bool,
    pub intercompany_balancing_account: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Accounting Method Create/Update Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingMethodRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub application: String,
    pub transaction_type: String,
    pub event_class: Option<String>,
    pub auto_accounting: Option<bool>,
    pub allow_manual_entries: Option<bool>,
    pub apply_rounding: Option<bool>,
    pub rounding_account_code: Option<String>,
    pub rounding_threshold: Option<String>,
    pub require_balancing: Option<bool>,
    pub intercompany_balancing_account: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Accounting Derivation Rule
/// Rules for deriving account codes from transaction attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountingDerivationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub accounting_method_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Line type: 'debit', 'credit', 'tax', 'discount'
    pub line_type: String,
    /// Priority (lower = higher priority)
    pub priority: i32,
    /// Conditions for rule activation
    pub conditions: serde_json::Value,
    /// Source field from the transaction
    pub source_field: Option<String>,
    /// Derivation type: 'constant', 'lookup', 'formula'
    pub derivation_type: String,
    /// Fixed account code (for 'constant' type)
    pub fixed_account_code: Option<String>,
    /// Lookup table (for 'lookup' type)
    pub account_derivation_lookup: serde_json::Value,
    /// Formula expression (for 'formula' type)
    pub formula_expression: Option<String>,
    pub sequence: i32,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subledger Journal Entry
/// The accounting representation of a subledger transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubledgerJournalEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Source subledger: 'payables', 'receivables', 'expenses', etc.
    pub source_application: String,
    /// Transaction type: 'invoice', 'payment', etc.
    pub source_transaction_type: String,
    /// ID of the source transaction
    pub source_transaction_id: Uuid,
    pub source_transaction_number: Option<String>,
    /// Accounting method applied
    pub accounting_method_id: Option<Uuid>,
    /// Journal entry number (auto-generated)
    pub entry_number: String,
    pub description: Option<String>,
    pub reference_number: Option<String>,
    /// GL date
    pub accounting_date: chrono::NaiveDate,
    pub period_name: Option<String>,
    /// Currency info
    pub currency_code: String,
    pub entered_currency_code: String,
    pub currency_conversion_date: Option<chrono::NaiveDate>,
    pub currency_conversion_type: Option<String>,
    pub currency_conversion_rate: Option<String>,
    /// Totals
    pub total_debit: String,
    pub total_credit: String,
    pub entered_debit: String,
    pub entered_credit: String,
    /// Status: 'draft', 'accounted', 'posted', 'transferred', 'reversed', 'error'
    pub status: String,
    pub error_message: Option<String>,
    /// Balancing
    pub balancing_segment: Option<String>,
    pub is_balanced: bool,
    /// GL transfer tracking
    pub gl_transfer_status: String,
    pub gl_transfer_date: Option<DateTime<Utc>>,
    pub gl_journal_entry_id: Option<Uuid>,
    /// Reversal tracking
    pub is_reversal: bool,
    pub reversal_of_id: Option<Uuid>,
    pub reversal_reason: Option<String>,
    /// Audit
    pub created_by: Option<Uuid>,
    pub posted_by: Option<Uuid>,
    pub accounted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subledger Journal Line
/// Individual debit/credit line within a journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubledgerJournalLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub journal_entry_id: Uuid,
    pub line_number: i32,
    /// Line type: 'debit', 'credit', 'tax', 'discount', 'rounding'
    pub line_type: String,
    /// Account code
    pub account_code: String,
    pub account_description: Option<String>,
    /// Derivation rule that produced this line
    pub derivation_rule_id: Option<Uuid>,
    /// Amounts
    pub entered_amount: String,
    pub accounted_amount: String,
    /// Currency
    pub currency_code: String,
    pub conversion_date: Option<chrono::NaiveDate>,
    pub conversion_rate: Option<String>,
    /// Descriptive flexfield attributes
    pub attribute_category: Option<String>,
    pub attribute1: Option<String>,
    pub attribute2: Option<String>,
    pub attribute3: Option<String>,
    pub attribute4: Option<String>,
    pub attribute5: Option<String>,
    pub attribute6: Option<String>,
    pub attribute7: Option<String>,
    pub attribute8: Option<String>,
    pub attribute9: Option<String>,
    pub attribute10: Option<String>,
    /// Tax
    pub tax_code: Option<String>,
    pub tax_rate: Option<String>,
    pub tax_amount: Option<String>,
    /// Source reference
    pub source_line_id: Option<Uuid>,
    pub source_line_type: Option<String>,
    /// Reversal
    pub is_reversal_line: bool,
    pub reversal_of_line_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subledger Accounting Event
/// Audit trail of accounting events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub event_number: String,
    /// Event type: 'creation', 'modification', 'cancellation', 'reversal', 'posting', 'transfer'
    pub event_type: String,
    pub source_application: String,
    pub source_transaction_type: String,
    pub source_transaction_id: Uuid,
    pub journal_entry_id: Option<Uuid>,
    pub event_date: chrono::NaiveDate,
    /// Status: 'processed', 'error', 'skipped'
    pub event_status: String,
    pub description: Option<String>,
    pub error_message: Option<String>,
    pub processed_by: Option<Uuid>,
    pub processed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// GL Transfer Log
/// Tracks transfers of subledger entries to the General Ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlTransferLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transfer_number: String,
    pub transfer_date: DateTime<Utc>,
    pub from_period: Option<String>,
    /// Status: 'pending', 'in_progress', 'completed', 'failed', 'reversed'
    pub status: String,
    pub error_message: Option<String>,
    pub total_entries: i32,
    pub total_debit: String,
    pub total_credit: String,
    pub included_applications: serde_json::Value,
    pub transferred_by: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub entries: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subledger Accounting Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaDashboardSummary {
    pub total_entries: i32,
    pub draft_count: i32,
    pub accounted_count: i32,
    pub posted_count: i32,
    pub transferred_count: i32,
    pub reversed_count: i32,
    pub error_count: i32,
    pub total_debit: String,
    pub total_credit: String,
    pub entries_by_application: serde_json::Value,
    pub entries_by_status: serde_json::Value,
    pub pending_transfer_count: i32,
    pub unbalanced_count: i32,
}

// ════════════════════════════════════════════════════════════════════════════════
// Encumbrance Management (Oracle Fusion GL > Encumbrance Management)
// ════════════════════════════════════════════════════════════════════════════════
//
// Tracks financial commitments before actual expenditure:
// - Requisitions → Preliminary encumbrances
// - Purchase Orders → Encumbrances (commitments)
// - Invoices → Partial/Full liquidation of encumbrances
// - Contracts → Long-term commitment tracking
//
// Supports budgetary control by reserving funds against budgets.

/// Encumbrance Type definition
/// Defines the types of commitments an organization tracks.
/// Oracle Fusion equivalent: GL > Encumbrance Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceType {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code (e.g., "PURCHASE_ORDER", "REQUISITION", "CONTRACT")
    pub code: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Category: "commitment", "obligation", "preliminary"
    pub category: String,
    /// Whether this encumbrance type is enabled
    pub is_enabled: bool,
    /// Whether this type can be manually entered
    pub allow_manual_entry: bool,
    /// Default encumbrance account code for this type
    pub default_encumbrance_account_code: Option<String>,
    /// Whether year-end carry-forward is allowed
    pub allow_carry_forward: bool,
    /// Priority for budget control (lower = checked first)
    pub priority: i32,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Encumbrance Entry header
/// Represents a commitment transaction (e.g., a purchase order creates an encumbrance).
/// Oracle Fusion equivalent: GL > Encumbrance Entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated entry number (e.g., "ENC-2024-00001")
    pub entry_number: String,
    /// Encumbrance type ID
    pub encumbrance_type_id: Uuid,
    /// Encumbrance type code (denormalized)
    pub encumbrance_type_code: String,
    /// Source document type (e.g., "purchase_order", "requisition", "contract")
    pub source_type: Option<String>,
    /// Source document ID
    pub source_id: Option<Uuid>,
    /// Source document number (e.g., PO-00123)
    pub source_number: Option<String>,
    /// Description/purpose of the encumbrance
    pub description: Option<String>,
    /// Encumbrance date (when the commitment was made)
    pub encumbrance_date: chrono::NaiveDate,
    /// Original encumbrance amount
    pub original_amount: String,
    /// Current remaining encumbrance amount
    pub current_amount: String,
    /// Amount that has been liquidated (matched to actual expenditure)
    pub liquidated_amount: String,
    /// Amount that has been manually adjusted
    pub adjusted_amount: String,
    /// Currency code
    pub currency_code: String,
    /// Status: "draft", "active", "partially_liquidated", "fully_liquidated", "cancelled", "expired"
    pub status: String,
    /// Budget period or fiscal year reference
    pub fiscal_year: Option<i32>,
    /// Period name
    pub period_name: Option<String>,
    /// Whether this entry has been carried forward from a prior year
    pub is_carry_forward: bool,
    /// Original entry ID if this is a carry-forward
    pub carried_forward_from_id: Option<Uuid>,
    /// Expiry date for time-limited commitments
    pub expiry_date: Option<chrono::NaiveDate>,
    /// Reference to the associated budget line (if applicable)
    pub budget_line_id: Option<Uuid>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Encumbrance Line
/// Individual line within an encumbrance entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entry_id: Uuid,
    /// Line number within the entry
    pub line_number: i32,
    /// Account code being encumbered
    pub account_code: String,
    /// Account description
    pub account_description: Option<String>,
    /// Department ID
    pub department_id: Option<Uuid>,
    /// Department name
    pub department_name: Option<String>,
    /// Project ID
    pub project_id: Option<Uuid>,
    /// Project name
    pub project_name: Option<String>,
    /// Cost center
    pub cost_center: Option<String>,
    /// Original encumbered amount
    pub original_amount: String,
    /// Current remaining amount
    pub current_amount: String,
    /// Liquidated amount
    pub liquidated_amount: String,
    /// Encumbrance account code (the account tracking the commitment)
    pub encumbrance_account_code: Option<String>,
    /// Source line reference
    pub source_line_id: Option<Uuid>,
    /// Descriptive flexfields
    pub attribute_category: Option<String>,
    pub attribute1: Option<String>,
    pub attribute2: Option<String>,
    pub attribute3: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Encumbrance Liquidation
/// Records the reduction of an encumbrance when actual expenditure occurs
/// (e.g., when an invoice is matched to a purchase order).
/// Oracle Fusion equivalent: GL > Encumbrance Liquidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceLiquidation {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated liquidation number
    pub liquidation_number: String,
    /// The encumbrance entry being liquidated
    pub encumbrance_entry_id: Uuid,
    /// Specific line being liquidated (None = header-level liquidation)
    pub encumbrance_line_id: Option<Uuid>,
    /// Liquidation type: "full", "partial", "final"
    pub liquidation_type: String,
    /// Amount being liquidated
    pub liquidation_amount: String,
    /// Source document type (e.g., "invoice", "payment", "journal_entry")
    pub source_type: Option<String>,
    /// Source document ID
    pub source_id: Option<Uuid>,
    /// Source document number
    pub source_number: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Liquidation date
    pub liquidation_date: chrono::NaiveDate,
    /// Status: "draft", "processed", "reversed"
    pub status: String,
    /// Reversal reference
    pub reversed_by_id: Option<Uuid>,
    pub reversal_reason: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Encumbrance Year-End Processing
/// Tracks carry-forward of open encumbrances to the next fiscal year.
/// Oracle Fusion equivalent: GL > Encumbrance Year-End Processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceCarryForward {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Processing batch number
    pub batch_number: String,
    /// Source fiscal year
    pub from_fiscal_year: i32,
    /// Target fiscal year
    pub to_fiscal_year: i32,
    /// Status: "draft", "processing", "completed", "reversed"
    pub status: String,
    /// Total number of entries carried forward
    pub entry_count: i32,
    /// Total amount carried forward
    pub total_amount: String,
    /// Description/notes
    pub description: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub processed_by: Option<Uuid>,
    pub processed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Encumbrance Dashboard Summary
/// Provides an overview of encumbrance activity for budgetary control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncumbranceSummary {
    /// Total active encumbrance amount
    pub total_active_amount: String,
    /// Total liquidated amount (period)
    pub total_liquidated_amount: String,
    /// Total adjusted amount (period)
    pub total_adjusted_amount: String,
    /// Count of active entries
    pub active_entry_count: i32,
    /// Count of entries by status
    pub entries_by_status: serde_json::Value,
    /// Count of entries by type
    pub entries_by_type: serde_json::Value,
    /// Breakdown by account code
    pub by_account: serde_json::Value,
    /// Breakdown by department
    pub by_department: serde_json::Value,
    /// Expiring soon count (next 30 days)
    pub expiring_soon_count: i32,
    /// Expiring soon amount
    pub expiring_soon_amount: String,
}

// Cash Position & Cash Forecasting (Oracle Fusion Treasury Management)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Treasury Management provides:
// - Cash Positions: Real-time view of cash balances across bank accounts
// - Cash Forecasts: Projected cash inflows and outflows over configurable periods
// - Forecast Sources: Configurable sources (AP, AR, Payroll, Purchasing, etc.)
// - Forecast Templates: Define forecast columns, time buckets, and aggregation
//
// Oracle Fusion equivalent: Financials > Treasury > Cash Management

/// Cash Position
/// Represents a snapshot of cash balances for a bank account at a point in time.
/// Oracle Fusion equivalent: Treasury > Cash Position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPosition {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Bank account ID (references reconciliation bank account)
    pub bank_account_id: Uuid,
    /// Bank account number (denormalized)
    pub account_number: String,
    /// Bank account name (denormalized)
    pub account_name: String,
    /// Currency code
    pub currency_code: String,
    /// Ledger (book) balance as of position date
    pub book_balance: String,
    /// Available balance (book balance minus holds/outstanding)
    pub available_balance: String,
    /// Float (deposits in transit not yet cleared)
    pub float_amount: String,
    /// One-day float (clearing next business day)
    pub one_day_float: String,
    /// Two-or-more day float
    pub two_day_float: String,
    /// Position date
    pub position_date: chrono::NaiveDate,
    /// Rolling average balance (e.g., 30-day)
    pub average_balance: Option<String>,
    /// Prior day closing balance
    pub prior_day_balance: Option<String>,
    /// Projected inflows for today
    pub projected_inflows: String,
    /// Projected outflows for today
    pub projected_outflows: String,
    /// Net projected change
    pub projected_net: String,
    /// Whether this position is reconciled
    pub is_reconciled: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cash Position Summary
/// Aggregated cash position across all bank accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashPositionSummary {
    pub organization_id: Uuid,
    /// Position date
    pub position_date: chrono::NaiveDate,
    /// Total book balance across all accounts
    pub total_book_balance: String,
    /// Total available balance
    pub total_available_balance: String,
    /// Total float
    pub total_float: String,
    /// Total projected inflows
    pub total_projected_inflows: String,
    /// Total projected outflows
    pub total_projected_outflows: String,
    /// Total net projected change
    pub total_projected_net: String,
    /// Number of bank accounts included
    pub account_count: i32,
    /// Breakdown by currency
    pub by_currency: serde_json::Value,
    /// Breakdown by bank account
    pub by_account: serde_json::Value,
}

/// Forecast Template
/// Defines the structure of a cash forecast (columns, time periods, sources).
/// Oracle Fusion equivalent: Treasury > Cash Forecast > Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecastTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique template code
    pub code: String,
    /// Template name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Time bucket type: "daily", "weekly", "monthly"
    pub bucket_type: String,
    /// Number of periods to forecast
    pub number_of_periods: i32,
    /// From-date offset (e.g., 0 = today, -7 = a week ago)
    pub start_offset_days: i32,
    /// Whether this is the default template
    pub is_default: bool,
    /// Whether the template is active
    pub is_active: bool,
    /// Template columns definition (JSON array of column configs)
    pub columns: serde_json::Value,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Forecast Source
/// Defines a data source that feeds into cash forecasts.
/// Oracle Fusion equivalent: Treasury > Cash Forecast > Sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecastSource {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Template ID this source belongs to
    pub template_id: Uuid,
    /// Source code (unique within template)
    pub code: String,
    /// Source name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Source type: "accounts_payable", "accounts_receivable", "payroll",
    ///             "purchasing", "manual", "budget", "intercompany"
    pub source_type: String,
    /// Cash flow direction: "inflow", "outflow", "both"
    pub cash_flow_direction: String,
    /// Whether this source is for actuals or forecasts
    pub is_actual: bool,
    /// Priority for display ordering
    pub display_order: i32,
    /// Whether the source is active
    pub is_active: bool,
    /// Lead time in days (expected delay between transaction and cash impact)
    pub lead_time_days: i32,
    /// Payment terms reference or description
    pub payment_terms_reference: Option<String>,
    /// GL account code filter (optional)
    pub account_code_filter: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cash Forecast
/// A specific forecast run generated from a template.
/// Oracle Fusion equivalent: Treasury > Cash Forecast > Forecasts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecast {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Forecast number (auto-generated)
    pub forecast_number: String,
    /// Template used to generate this forecast
    pub template_id: Uuid,
    /// Template name (denormalized)
    pub template_name: String,
    /// Forecast name/description
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Start date of the forecast period
    pub start_date: chrono::NaiveDate,
    /// End date of the forecast period
    pub end_date: chrono::NaiveDate,
    /// Opening balance (actual balance at start date)
    pub opening_balance: String,
    /// Total projected inflows
    pub total_inflows: String,
    /// Total projected outflows
    pub total_outflows: String,
    /// Net cash flow
    pub net_cash_flow: String,
    /// Closing projected balance
    pub closing_balance: String,
    /// Minimum balance encountered during the period
    pub minimum_balance: String,
    /// Maximum balance encountered during the period
    pub maximum_balance: String,
    /// Deficit periods (where balance falls below threshold)
    pub deficit_count: i32,
    /// Surplus periods
    pub surplus_count: i32,
    /// Status: "draft", "generated", "approved", "superseded"
    pub status: String,
    /// Whether this is the latest forecast for this template
    pub is_latest: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cash Forecast Line
/// Individual line within a forecast, representing a source for a time period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecastLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Forecast header ID
    pub forecast_id: Uuid,
    /// Forecast source ID
    pub source_id: Uuid,
    /// Source name (denormalized)
    pub source_name: String,
    /// Source type (denormalized)
    pub source_type: String,
    /// Cash flow direction
    pub cash_flow_direction: String,
    /// Period start date
    pub period_start_date: chrono::NaiveDate,
    /// Period end date
    pub period_end_date: chrono::NaiveDate,
    /// Period label (e.g., "Week 3", "Mar 2025")
    pub period_label: String,
    /// Period sequence number
    pub period_sequence: i32,
    /// Amount for this period
    pub amount: String,
    /// Running cumulative amount
    pub cumulative_amount: String,
    /// Whether this is actual data (vs projected)
    pub is_actual: bool,
    /// Currency code
    pub currency_code: String,
    /// Number of underlying transactions
    pub transaction_count: i32,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cash Forecast Summary
/// Summary view of a cash forecast for dashboard display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashForecastSummary {
    /// Template ID
    pub template_id: Uuid,
    /// Template name
    pub template_name: String,
    /// Forecast ID
    pub forecast_id: Uuid,
    /// Forecast number
    pub forecast_number: String,
    /// Start date
    pub start_date: chrono::NaiveDate,
    /// End date
    pub end_date: chrono::NaiveDate,
    /// Opening balance
    pub opening_balance: String,
    /// Total inflows
    pub total_inflows: String,
    /// Total outflows
    pub total_outflows: String,
    /// Net cash flow
    pub net_cash_flow: String,
    /// Closing balance
    pub closing_balance: String,
    /// Minimum balance
    pub minimum_balance: String,
    /// Deficit count
    pub deficit_count: i32,
    /// Surplus count
    pub surplus_count: i32,
    /// Inflows by source (for chart)
    pub inflows_by_source: serde_json::Value,
    /// Outflows by source (for chart)
    pub outflows_by_source: serde_json::Value,
    /// Balance trend (array of period-end balances)
    pub balance_trend: serde_json::Value,
}

// ════════════════════════════════════════════════════════════════════════════════
// Procurement Sourcing Management (Oracle Fusion SCM > Procurement > Sourcing)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Procurement Sourcing provides:
// - Sourcing Events: RFQs (Request for Quote), RFPs (Request for Proposal), RFI
// - Supplier Responses: Bids/quotation submissions with line-level pricing
// - Scoring & Evaluation: Weighted scoring criteria, team evaluation
// - Award: Best-value analysis, split awards, multi-supplier awards
// - Templates: Reusable sourcing templates for recurring procurement
// - Negotiation: Multi-round negotiation with suppliers
//
// Oracle Fusion equivalent: Procurement > Sourcing > Negotiations

/// Sourcing Event (Negotiation)
/// Represents an RFQ, RFP, or other sourcing event sent to suppliers.
/// Oracle Fusion equivalent: Procurement > Sourcing > Negotiations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated event number (e.g., "SE-2024-00001")
    pub event_number: String,
    /// Event title/subject
    pub title: String,
    /// Description of what is being sourced
    pub description: Option<String>,
    /// Event type: "rfq" (Request for Quote), "rfp" (Request for Proposal),
    /// "rfi" (Request for Information), "auction" (Reverse Auction)
    pub event_type: String,
    /// Status: "draft", "published", "response_open", "evaluation", "awarded",
    ///         "cancelled", "closed"
    pub status: String,
    /// Style: "sealed" (blind bidding), "open" (visible bids), "reverse_auction"
    pub style: String,
    /// Deadline for supplier responses
    pub response_deadline: chrono::NaiveDate,
    /// When the event was published
    pub published_at: Option<DateTime<Utc>>,
    /// When the event was closed/awarded
    pub closed_at: Option<DateTime<Utc>>,
    /// Currency for all pricing
    pub currency_code: String,
    /// Sourcing template reference
    pub template_id: Option<Uuid>,
    /// Template name (denormalized)
    pub template_name: Option<String>,
    /// Evaluation team lead
    pub evaluation_lead_id: Option<Uuid>,
    pub evaluation_lead_name: Option<String>,
    /// Scoring method: "weighted", "pass_fail", "manual", "lowest_price"
    pub scoring_method: String,
    /// Whether supplier responses are visible to other suppliers
    pub are_bids_visible: bool,
    /// Allow suppliers to see their rank
    pub allow_supplier_rank_visibility: bool,
    /// Contact for inquiries
    pub contact_person_id: Option<Uuid>,
    pub contact_person_name: Option<String>,
    /// Terms and conditions
    pub terms_and_conditions: Option<String>,
    /// Attachment references
    pub attachments: serde_json::Value,
    /// Number of invited suppliers
    pub invited_supplier_count: i32,
    /// Number of suppliers who responded
    pub response_count: i32,
    /// Award summary
    pub award_summary: serde_json::Value,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub cancelled_by: Option<Uuid>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Event Line
/// Individual items or services being sourced within an event.
/// Oracle Fusion equivalent: Negotiation Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingEventLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent sourcing event
    pub event_id: Uuid,
    /// Line number within the event
    pub line_number: i32,
    /// Item / service description
    pub description: String,
    /// Item number / SKU reference
    pub item_number: Option<String>,
    /// Item category
    pub category: Option<String>,
    /// Quantity being sourced
    pub quantity: String,
    /// Unit of measure (e.g., "EA", "KG", "LOT")
    pub uom: String,
    /// Target / estimated unit price
    pub target_price: Option<String>,
    /// Target / estimated total price
    pub target_total: Option<String>,
    /// Required delivery date
    pub need_by_date: Option<chrono::NaiveDate>,
    /// Ship-to location
    pub ship_to: Option<String>,
    /// Technical specifications
    pub specifications: Option<serde_json::Value>,
    /// Whether partial quantity bids are allowed
    pub allow_partial_quantity: bool,
    /// Minimum award quantity (for split awards)
    pub min_award_quantity: Option<String>,
    /// Line status: "open", "awarded", "cancelled"
    pub status: String,
    /// Awarded supplier ID (after award)
    pub awarded_supplier_id: Option<Uuid>,
    /// Awarded supplier name (denormalized)
    pub awarded_supplier_name: Option<String>,
    /// Awarded unit price (after award)
    pub awarded_price: Option<String>,
    /// Awarded quantity (for split awards)
    pub awarded_quantity: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Event Invite
/// Tracks which suppliers are invited to participate in a sourcing event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingInvite {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Sourcing event reference
    pub event_id: Uuid,
    /// Supplier reference
    pub supplier_id: Uuid,
    /// Supplier name (denormalized)
    pub supplier_name: Option<String>,
    /// Supplier email for notifications
    pub supplier_email: Option<String>,
    /// Whether the supplier has viewed the event
    pub is_viewed: bool,
    pub viewed_at: Option<DateTime<Utc>>,
    /// Whether the supplier has responded
    pub has_responded: bool,
    pub responded_at: Option<DateTime<Utc>>,
    /// Status: "invited", "viewed", "responded", "declined", "disqualified"
    pub status: String,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Response (Bid/Quotation)
/// A supplier's response to a sourcing event with line-level pricing.
/// Oracle Fusion equivalent: Supplier Response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Sourcing event reference
    pub event_id: Uuid,
    /// Response number (auto-generated)
    pub response_number: String,
    /// Supplier reference
    pub supplier_id: Uuid,
    /// Supplier name (denormalized)
    pub supplier_name: Option<String>,
    /// Status: "draft", "submitted", "under_review", "shortlisted", "rejected",
    ///         "awarded", "disqualified"
    pub status: String,
    /// Total bid amount (sum of all line amounts)
    pub total_amount: String,
    /// Total score (after evaluation)
    pub total_score: Option<String>,
    /// Rank among all responses (after evaluation)
    pub rank: Option<i32>,
    /// Whether this response meets all requirements
    pub is_compliant: Option<bool>,
    /// Supplier notes / cover letter
    pub cover_letter: Option<String>,
    /// Validity date for the bid
    pub valid_until: Option<chrono::NaiveDate>,
    /// Payment terms offered
    pub payment_terms: Option<String>,
    /// Delivery lead time in days
    pub lead_time_days: Option<i32>,
    /// Warranty offered (months)
    pub warranty_months: Option<i32>,
    /// Attachment references
    pub attachments: serde_json::Value,
    /// Evaluation notes
    pub evaluation_notes: Option<String>,
    /// Submitted at
    pub submitted_at: Option<DateTime<Utc>>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub evaluated_by: Option<Uuid>,
    pub evaluated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Response Line
/// Individual line pricing within a supplier's response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierResponseLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Response header reference
    pub response_id: Uuid,
    /// Sourcing event line reference
    pub event_line_id: Uuid,
    /// Line number
    pub line_number: i32,
    /// Quoted unit price
    pub unit_price: String,
    /// Quoted quantity
    pub quantity: String,
    /// Total line amount (unit_price × quantity)
    pub line_amount: String,
    /// Discount percentage offered
    pub discount_percent: Option<String>,
    /// Effective price after discount
    pub effective_price: Option<String>,
    /// Promised delivery date
    pub promised_delivery_date: Option<chrono::NaiveDate>,
    /// Lead time in days
    pub lead_time_days: Option<i32>,
    /// Whether this line meets specifications
    pub is_compliant: Option<bool>,
    /// Line score (after evaluation)
    pub score: Option<String>,
    /// Notes from the supplier
    pub supplier_notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Scoring Criterion
/// Defines an evaluation criterion for scoring supplier responses.
/// Oracle Fusion equivalent: Negotiation > Requirements & Scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoringCriterion {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Sourcing event reference
    pub event_id: Uuid,
    /// Criterion name (e.g., "Price", "Quality", "Delivery Time", "Technical Fit")
    pub name: String,
    /// Description of how to evaluate this criterion
    pub description: Option<String>,
    /// Weight in total score (0-100, all criteria should sum to 100)
    pub weight: String,
    /// Maximum possible score
    pub max_score: String,
    /// Criterion type: "price", "quality", "delivery", "technical", "compliance", "custom"
    pub criterion_type: String,
    /// Display order
    pub display_order: i32,
    /// Whether this is a mandatory (knockout) criterion
    pub is_mandatory: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Score given to a specific response for a specific criterion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseScore {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Response reference
    pub response_id: Uuid,
    /// Scoring criterion reference
    pub criterion_id: Uuid,
    /// Score given (0 to max_score)
    pub score: String,
    /// Weighted score (score × weight / 100)
    pub weighted_score: String,
    /// Evaluator's notes
    pub notes: Option<String>,
    /// Evaluator
    pub scored_by: Option<Uuid>,
    pub scored_at: Option<DateTime<Utc>>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Award
/// Records the award decision for a sourcing event.
/// Oracle Fusion equivalent: Negotiation > Award
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingAward {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Sourcing event reference
    pub event_id: Uuid,
    /// Award number (auto-generated)
    pub award_number: String,
    /// Status: "pending", "approved", "rejected", "cancelled"
    pub status: String,
    /// Award method: "single", "split", "best_value", "lowest_price"
    pub award_method: String,
    /// Total awarded amount
    pub total_awarded_amount: String,
    /// Reason for award decision
    pub award_rationale: Option<String>,
    /// Awarded by
    pub awarded_by: Option<Uuid>,
    pub awarded_at: Option<DateTime<Utc>>,
    /// Approved by
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    /// Rejected reason
    pub rejected_reason: Option<String>,
    /// Award lines (supplier-level awards)
    pub lines: serde_json::Value,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Award Line
/// Individual line in an award (maps event lines to winning suppliers).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingAwardLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Award header reference
    pub award_id: Uuid,
    /// Sourcing event line reference
    pub event_line_id: Uuid,
    /// Winning supplier response reference
    pub response_id: Uuid,
    /// Winning supplier
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    /// Awarded quantity
    pub awarded_quantity: String,
    /// Awarded unit price
    pub awarded_unit_price: String,
    /// Awarded total amount
    pub awarded_amount: String,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Template
/// Reusable template for creating sourcing events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Template code
    pub code: String,
    /// Template name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Default event type
    pub default_event_type: String,
    /// Default style
    pub default_style: String,
    /// Default scoring method
    pub default_scoring_method: String,
    /// Default response deadline offset (days from publish)
    pub default_response_deadline_days: i32,
    /// Default currency
    pub currency_code: String,
    /// Whether bids are visible by default
    pub default_bids_visible: bool,
    /// Default terms and conditions
    pub default_terms: Option<String>,
    /// Predefined scoring criteria (JSON array)
    pub default_scoring_criteria: serde_json::Value,
    /// Predefined line templates (JSON array)
    pub default_lines: serde_json::Value,
    /// Is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sourcing Dashboard Summary
/// Overview of sourcing activity for the procurement dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcingSummary {
    /// Total active events
    pub active_event_count: i32,
    /// Total draft events
    pub draft_event_count: i32,
    /// Events pending evaluation
    pub pending_evaluation_count: i32,
    /// Events awarded (period)
    pub awarded_event_count: i32,
    /// Total awarded value (period)
    pub total_awarded_value: String,
    /// Average savings percentage vs target price
    pub average_savings_percent: String,
    /// Events by status
    pub events_by_status: serde_json::Value,
    /// Events by type
    pub events_by_type: serde_json::Value,
    /// Top suppliers by award value
    pub top_suppliers: serde_json::Value,
    /// Upcoming deadlines
    pub upcoming_deadlines: serde_json::Value,
}

// ════════════════════════════════════════════════════════════════════════════════
// Lease Accounting (ASC 842 / IFRS 16)
// Oracle Fusion Cloud ERP: Financials > Lease Management
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Lease Management provides:
// - Lease Contracts: Track lease agreements with classification (operating/finance)
// - Right-of-Use (ROU) Assets: Asset recognition for leased assets
// - Lease Liability: Present value of future lease payments
// - Amortization Schedules: Liability amortization and asset depreciation
// - Lease Payments: Payment schedules with escalation/renewal terms
// - Lease Modifications: Accounting for changes to lease terms
// - Lease Impairment: ROU asset impairment review
// - Lease Termination: Early termination accounting
//
// Oracle Fusion equivalent: Financials > Lease Management

/// Lease Accounting Method
/// ASC 842 distinguishes between operating and finance leases for lessees.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LeaseClassification {
    #[default]
    Operating,
    Finance,
}

/// Lease contract header
/// Represents a lease agreement between a lessee and lessor.
/// Oracle Fusion equivalent: Lease Management > Lease Contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseContract {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated lease number (e.g., "LSE-2024-00001")
    pub lease_number: String,
    /// Lease title / description
    pub title: String,
    pub description: Option<String>,
    /// Classification: "operating" or "finance"
    pub classification: String,
    /// Lessor / supplier information
    pub lessor_id: Option<Uuid>,
    pub lessor_name: Option<String>,
    /// Asset being leased
    pub asset_description: Option<String>,
    /// Asset location
    pub location: Option<String>,
    /// Department
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    /// Lease dates
    pub commencement_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    /// Lease term in months
    pub lease_term_months: i32,
    /// Whether the lease includes a purchase option
    pub purchase_option_exists: bool,
    /// Whether the purchase option is reasonably certain to be exercised
    pub purchase_option_likely: bool,
    /// Whether there is a renewal option
    pub renewal_option_exists: bool,
    /// Renewal option term in months
    pub renewal_option_months: Option<i32>,
    /// Whether renewal is reasonably certain to be exercised
    pub renewal_option_likely: bool,
    /// Discount rate (incremental borrowing rate)
    pub discount_rate: String,
    /// Currency
    pub currency_code: String,
    /// Payment frequency: "monthly", "quarterly", "annually"
    pub payment_frequency: String,
    /// Annual escalation rate percentage
    pub escalation_rate: Option<String>,
    /// Escalation frequency in months (e.g., 12 for annual)
    pub escalation_frequency_months: Option<i32>,
    /// Financial summary
    pub total_lease_payments: String,
    pub initial_lease_liability: String,
    pub initial_rou_asset_value: String,
    pub residual_guarantee_amount: Option<String>,
    /// Current balances
    pub current_lease_liability: String,
    pub current_rou_asset_value: String,
    pub accumulated_rou_depreciation: String,
    /// Payment tracking
    pub total_payments_made: String,
    pub periods_elapsed: i32,
    /// GL account codes
    pub rou_asset_account_code: Option<String>,
    pub rou_depreciation_account_code: Option<String>,
    pub lease_liability_account_code: Option<String>,
    pub lease_expense_account_code: Option<String>,
    pub interest_expense_account_code: Option<String>,
    /// Status: "draft", "active", "modified", "impaired", "terminated", "expired"
    pub status: String,
    /// Impairment tracking
    pub impairment_amount: Option<String>,
    pub impairment_date: Option<chrono::NaiveDate>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lease payment schedule line
/// Oracle Fusion equivalent: Lease Management > Payment Schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeasePayment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub lease_id: Uuid,
    /// Period sequence number
    pub period_number: i32,
    /// Payment due date
    pub payment_date: chrono::NaiveDate,
    /// Total payment amount
    pub payment_amount: String,
    /// Interest portion of the payment
    pub interest_amount: String,
    /// Principal portion of the payment
    pub principal_amount: String,
    /// Remaining lease liability after this payment
    pub remaining_liability: String,
    /// ROU asset value after depreciation for this period
    pub rou_asset_value: String,
    /// ROU depreciation for this period
    pub rou_depreciation: String,
    /// Accumulated ROU depreciation after this period
    pub accumulated_depreciation: String,
    /// Straight-line lease expense (for operating leases)
    pub lease_expense: String,
    /// Whether this payment has been made
    pub is_paid: bool,
    /// Payment reference
    pub payment_reference: Option<String>,
    /// GL journal entry reference
    pub journal_entry_id: Option<Uuid>,
    /// Status: "scheduled", "paid", "overdue", "cancelled"
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lease modification
/// Tracks changes to lease terms that require remeasurement.
/// Oracle Fusion equivalent: Lease Management > Modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseModification {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub lease_id: Uuid,
    /// Modification number (sequential)
    pub modification_number: i32,
    /// Type: "term_extension", "scope_change", "payment_change", "rate_change",
    ///       "reclassification"
    pub modification_type: String,
    /// Description of the modification
    pub description: Option<String>,
    /// Effective date of the modification
    pub effective_date: chrono::NaiveDate,
    /// Previous lease term (months)
    pub previous_term_months: Option<i32>,
    /// New lease term (months)
    pub new_term_months: Option<i32>,
    /// Previous end date
    pub previous_end_date: Option<chrono::NaiveDate>,
    /// New end date
    pub new_end_date: Option<chrono::NaiveDate>,
    /// Previous discount rate
    pub previous_discount_rate: Option<String>,
    /// New discount rate
    pub new_discount_rate: Option<String>,
    /// Change in lease liability due to modification
    pub liability_adjustment: String,
    /// Change in ROU asset due to modification
    pub rou_asset_adjustment: String,
    /// Status: "pending", "processed", "reversed"
    pub status: String,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lease termination
/// Oracle Fusion equivalent: Lease Management > Termination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseTermination {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub lease_id: Uuid,
    /// Termination type: "early", "end_of_term", "mutual_agreement", "default"
    pub termination_type: String,
    /// Termination date
    pub termination_date: chrono::NaiveDate,
    /// Reason for termination
    pub reason: Option<String>,
    /// Remaining lease liability at termination
    pub remaining_liability: String,
    /// ROU asset value at termination (net of depreciation)
    pub remaining_rou_asset: String,
    /// Termination penalty / fee
    pub termination_penalty: String,
    /// Gain/loss on termination
    pub gain_loss_amount: String,
    /// "gain" or "loss"
    pub gain_loss_type: Option<String>,
    /// GL journal entry reference
    pub journal_entry_id: Option<Uuid>,
    /// Status: "pending", "processed", "reversed"
    pub status: String,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lease Accounting Dashboard Summary
/// Oracle Fusion equivalent: Lease Management Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseDashboardSummary {
    pub total_active_leases: i32,
    pub total_lease_liability: String,
    pub total_rou_assets: String,
    pub total_rou_depreciation: String,
    pub total_net_rou_assets: String,
    pub total_payments_made: String,
    pub operating_lease_count: i32,
    pub finance_lease_count: i32,
    pub upcoming_payments_count: i32,
    pub upcoming_payments_amount: String,
    pub leases_expiring_90_days: i32,
    pub leases_by_classification: serde_json::Value,
    pub leases_by_status: serde_json::Value,
    pub liability_by_period: serde_json::Value,
}

// ════════════════════════════════════════════════════════════════════════════════
// Project Costing (Oracle Fusion Cloud ERP: Project Management > Project Costing)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Project Costing provides:
// - Cost Transactions: Track labor, material, expense, and other costs against projects/tasks
// - Burden Schedules: Define overhead/burden rate schedules for cost types
// - Cost Burdening: Apply burden rates to raw costs to compute burdened amounts
// - Cost Adjustments: Adjust previously recorded costs (increase, decrease, transfer)
// - Cost Distributions: Distribute project costs to GL accounts
// - Capitalization: Capitalize eligible project costs as fixed assets
// - Cost Reporting: Dashboard with cost breakdowns by project, type, and period
//
// Oracle Fusion equivalent: Project Management > Project Costing

/// Project cost transaction
/// Records a cost incurred against a project/task.
/// Oracle Fusion: Project Costing > Cost Transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCostTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated transaction number (e.g., "PJC-2024-00001")
    pub transaction_number: String,
    /// Project reference
    pub project_id: Uuid,
    /// Project number (denormalized for display)
    pub project_number: Option<String>,
    /// Task reference (optional - cost can be at project level)
    pub task_id: Option<Uuid>,
    /// Task number (denormalized)
    pub task_number: Option<String>,
    /// Cost type: "labor", "material", "expense", "equipment", "other"
    pub cost_type: String,
    /// Raw cost amount (before burdening)
    pub raw_cost_amount: String,
    /// Burdened cost amount (raw + burden)
    pub burdened_cost_amount: String,
    /// Burden amount (overhead applied)
    pub burden_amount: String,
    /// Currency code
    pub currency_code: String,
    /// Transaction date (when cost was incurred)
    pub transaction_date: chrono::NaiveDate,
    /// GL posting date
    pub gl_date: Option<chrono::NaiveDate>,
    /// Description of the cost
    pub description: Option<String>,
    /// Supplier/vendor reference (for material/expense costs)
    pub supplier_id: Option<Uuid>,
    /// Supplier name (denormalized)
    pub supplier_name: Option<String>,
    /// Employee reference (for labor costs)
    pub employee_id: Option<Uuid>,
    /// Employee name (denormalized)
    pub employee_name: Option<String>,
    /// Expenditure type / category
    pub expenditure_category: Option<String>,
    /// Quantity (hours for labor, units for material)
    pub quantity: Option<String>,
    /// Unit of measure ("hours", "each", "lot")
    pub unit_of_measure: Option<String>,
    /// Rate per unit
    pub unit_rate: Option<String>,
    /// Billable flag (whether this cost can be billed to customer)
    pub is_billable: bool,
    /// Capitalizable flag (whether this cost can be capitalized as an asset)
    pub is_capitalizable: bool,
    /// Status: "draft", "approved", "distributed", "adjusted", "reversed", "capitalized"
    pub status: String,
    /// GL distribution reference
    pub distribution_id: Option<Uuid>,
    /// Original transaction reference (for adjustments)
    pub original_transaction_id: Option<Uuid>,
    /// Adjustment type (if this is an adjustment): "increase", "decrease", "transfer"
    pub adjustment_type: Option<String>,
    /// Adjustment reason
    pub adjustment_reason: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Burden schedule definition
/// Defines overhead/burden rates to be applied to project costs.
/// Oracle Fusion: Project Costing > Burden Schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurdenSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Schedule code (e.g., "OH-STD-2024")
    pub code: String,
    /// Schedule name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Status: "draft", "active", "inactive"
    pub status: String,
    /// Effective from date
    pub effective_from: chrono::NaiveDate,
    /// Effective to date
    pub effective_to: Option<chrono::NaiveDate>,
    /// Whether this is the default schedule for the organization
    pub is_default: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Burden schedule line (maps cost type to burden rate)
/// Oracle Fusion: Project Costing > Burden Schedule Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurdenScheduleLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent schedule reference
    pub schedule_id: Uuid,
    /// Line number within schedule
    pub line_number: i32,
    /// Cost type this line applies to: "labor", "material", "expense", "equipment", "other"
    pub cost_type: String,
    /// Expenditure category filter (None = applies to all of this cost type)
    pub expenditure_category: Option<String>,
    /// Burden rate percentage (e.g., "25.00" means 25% overhead)
    pub burden_rate_percent: String,
    /// GL account code for the burden amount
    pub burden_account_code: Option<String>,
    /// Whether this line is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cost adjustment
/// Records adjustments to previously recorded cost transactions.
/// Oracle Fusion: Project Costing > Cost Adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCostAdjustment {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated adjustment number
    pub adjustment_number: String,
    /// Original cost transaction being adjusted
    pub original_transaction_id: Uuid,
    /// Adjustment type: "increase", "decrease", "transfer", "reversal"
    pub adjustment_type: String,
    /// Adjustment amount (absolute value)
    pub adjustment_amount: String,
    /// New total raw cost after adjustment
    pub new_raw_cost: String,
    /// New burdened cost after adjustment
    pub new_burdened_cost: String,
    /// Reason for the adjustment
    pub reason: String,
    /// Description
    pub description: Option<String>,
    /// Effective date of the adjustment
    pub effective_date: chrono::NaiveDate,
    /// For transfers: destination project
    pub transfer_to_project_id: Option<Uuid>,
    /// For transfers: destination task
    pub transfer_to_task_id: Option<Uuid>,
    /// Status: "pending", "approved", "rejected", "processed"
    pub status: String,
    /// Created cost transaction (the adjustment transaction)
    pub created_transaction_id: Option<Uuid>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cost distribution line (GL posting for a cost transaction)
/// Oracle Fusion: Project Costing > Cost Distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCostDistribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Source cost transaction
    pub transaction_id: Uuid,
    /// Distribution line number
    pub line_number: i32,
    /// GL account code for the debit
    pub debit_account_code: String,
    /// GL account code for the credit
    pub credit_account_code: String,
    /// Distribution amount
    pub amount: String,
    /// Distribution type: "raw_cost", "burden", "total"
    pub distribution_type: String,
    /// GL posting date
    pub gl_date: chrono::NaiveDate,
    /// Whether this has been posted to GL
    pub is_posted: bool,
    /// GL batch reference
    pub gl_batch_id: Option<Uuid>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Project Costing Dashboard Summary
/// Oracle Fusion: Project Costing Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCostingSummary {
    /// Total number of projects with costs
    pub project_count: i32,
    /// Total raw costs across all projects
    pub total_raw_costs: String,
    /// Total burdened costs across all projects
    pub total_burdened_costs: String,
    /// Total burden (overhead) across all projects
    pub total_burden: String,
    /// Total capitalized costs
    pub total_capitalized: String,
    /// Total billed to customers
    pub total_billed: String,
    /// Costs by type breakdown
    pub costs_by_type: serde_json::Value,
    /// Costs by project breakdown (top projects)
    pub costs_by_project: serde_json::Value,
    /// Costs by month trend
    pub costs_by_month: serde_json::Value,
    /// Pending adjustments count
    pub pending_adjustments: i32,
    /// Pending distributions count
    pub pending_distributions: i32,
}

// ════════════════════════════════════════════════════════════════════════════════
// Cost Allocations (Oracle Fusion GL > Cost Allocation / Mass Allocations)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Cost Allocations provide:
// - Allocation Pools: Define cost pools (groups of accounts) to be allocated
// - Allocation Bases: Statistical or financial bases for distribution
//   (e.g., headcount, square footage, revenue, direct costs)
// - Allocation Rules: Map pools to target cost centers using bases
// - Rule Versions: Versioned rules with effective dates and approval workflow
// - Rule Execution: Run allocations to generate journal entries
// - Recurring Schedules: Schedule periodic allocation runs
// - Allocation History: Audit trail of all allocation runs
//
// Oracle Fusion equivalent: Financials > General Ledger > Allocations

/// Cost allocation pool definition
/// A pool defines a group of cost accounts whose balances are to be distributed.
/// Oracle Fusion equivalent: GL > Allocations > Cost Pools
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationPool {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique pool code (e.g., "RENT_POOL", "IT_OVERHEAD")
    pub code: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Pool type: "cost_center", "project", "department", "custom"
    pub pool_type: String,
    /// Account code filter for selecting pool source balances
    /// (e.g., {"account_codes": ["6100", "6110"]})
    pub source_account_codes: serde_json::Value,
    /// Department filter for pool source (optional)
    pub source_department_id: Option<Uuid>,
    /// Cost center filter for pool source (optional)
    pub source_cost_center: Option<String>,
    /// Whether this pool is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation base definition
/// Defines the statistical or financial measure used to distribute costs.
/// Oracle Fusion equivalent: GL > Allocations > Allocation Bases
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationBase {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique base code (e.g., "HEADCOUNT", "SQFT", "REVENUE")
    pub code: String,
    /// Display name (e.g., "Employee Headcount", "Square Footage")
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Base type: "statistical" (user-entered), "financial" (from GL)
    pub base_type: String,
    /// For financial bases: the account code pattern to use as the base
    pub financial_account_code: Option<String>,
    /// Unit of measure (e.g., "persons", "sq_meters", "USD")
    pub unit_of_measure: Option<String>,
    /// Whether this base is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Statistical base value
/// User-entered or imported statistical measures per cost center/department.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationBaseValue {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Reference to the allocation base
    pub base_id: Uuid,
    /// Base code (denormalized)
    pub base_code: String,
    /// Dimension reference (department, cost center, or project)
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub cost_center: Option<String>,
    pub project_id: Option<Uuid>,
    /// The statistical value
    pub value: String,
    /// Effective date of this value
    pub effective_date: chrono::NaiveDate,
    /// Source: "manual", "import", "system"
    pub source: String,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation rule definition
/// Maps a cost pool to target cost centers using an allocation base.
/// Oracle Fusion equivalent: GL > Allocations > Allocation Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated rule number (e.g., "ALLOC-001")
    pub rule_number: String,
    /// Rule name (e.g., "Rent Allocation by SQFT")
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Reference to the cost pool
    pub pool_id: Uuid,
    /// Pool code (denormalized)
    pub pool_code: String,
    /// Reference to the allocation base
    pub base_id: Uuid,
    /// Base code (denormalized)
    pub base_code: String,
    /// Allocation method: "proportional", "fixed_percent", "fixed_amount"
    pub allocation_method: String,
    /// Journal entry description template
    pub journal_description: Option<String>,
    /// Offset (contra) account for the credit side of the allocation
    pub offset_account_code: Option<String>,
    /// Rule status: "draft", "active", "inactive"
    pub status: String,
    /// Current version number
    pub current_version: i32,
    /// Effective dates
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    /// Whether this rule generates reversing entries
    pub is_reversing: bool,
    /// Currency code
    pub currency_code: String,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation rule target line
/// Defines a target cost center and optional fixed percentage/amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationRuleTarget {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Rule reference
    pub rule_id: Uuid,
    /// Line number
    pub line_number: i32,
    /// Target department
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    /// Target cost center
    pub cost_center: Option<String>,
    /// Target project
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    /// Target debit account code (where the allocated cost goes)
    pub target_account_code: String,
    /// Fixed percentage (for "fixed_percent" method)
    pub fixed_percent: Option<String>,
    /// Fixed amount (for "fixed_amount" method)
    pub fixed_amount: Option<String>,
    /// Whether this target line is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation run (execution)
/// Represents a single execution of an allocation rule.
/// Oracle Fusion equivalent: GL > Allocations > Run Allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated run number (e.g., "ARUN-2024-00001")
    pub run_number: String,
    /// Rule reference
    pub rule_id: Uuid,
    /// Rule name (denormalized)
    pub rule_name: String,
    /// Rule number (denormalized)
    pub rule_number: String,
    /// Allocation period start
    pub period_start: chrono::NaiveDate,
    /// Allocation period end
    pub period_end: chrono::NaiveDate,
    /// Total source amount (from pool)
    pub total_source_amount: String,
    /// Total allocated amount
    pub total_allocated_amount: String,
    /// Number of target lines generated
    pub line_count: i32,
    /// Status: "draft", "posted", "reversed"
    pub status: String,
    /// Journal entry reference (the generated GL batch)
    pub journal_entry_id: Option<Uuid>,
    /// Run date
    pub run_date: chrono::NaiveDate,
    /// Reversal reference
    pub reversed_by_id: Option<Uuid>,
    pub reversal_reason: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation run line (individual debit/credit)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationRunLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Run header reference
    pub run_id: Uuid,
    /// Line number
    pub line_number: i32,
    /// Line type: "debit" (target), "credit" (offset/source)
    pub line_type: String,
    /// Account code
    pub account_code: String,
    /// Department ID
    pub department_id: Option<Uuid>,
    /// Department name
    pub department_name: Option<String>,
    /// Cost center
    pub cost_center: Option<String>,
    /// Project ID
    pub project_id: Option<Uuid>,
    /// Allocated amount
    pub amount: String,
    /// Base value used for this allocation
    pub base_value_used: Option<String>,
    /// Percentage of total base
    pub percent_of_total: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Allocation summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationSummary {
    /// Total active rules
    pub active_rule_count: i32,
    /// Total pools
    pub pool_count: i32,
    /// Total allocation runs (period)
    pub run_count: i32,
    /// Total allocated amount (period)
    pub total_allocated_amount: String,
    /// Runs by status
    pub runs_by_status: serde_json::Value,
    /// Allocations by pool
    pub allocations_by_pool: serde_json::Value,
    /// Top allocation rules by amount
    pub top_rules: serde_json::Value,
}

// ============================================================================
// Financial Reporting (Oracle Fusion GL > Financial Reporting Center)
// ============================================================================

/// Report template types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinancialReportType {
    TrialBalance,
    IncomeStatement,
    BalanceSheet,
    CashFlow,
    Custom,
}

impl std::fmt::Display for FinancialReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinancialReportType::TrialBalance => write!(f, "trial_balance"),
            FinancialReportType::IncomeStatement => write!(f, "income_statement"),
            FinancialReportType::BalanceSheet => write!(f, "balance_sheet"),
            FinancialReportType::CashFlow => write!(f, "cash_flow"),
            FinancialReportType::Custom => write!(f, "custom"),
        }
    }
}

/// Financial Report Template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// trial_balance, income_statement, balance_sheet, cash_flow, custom
    pub report_type: String,
    pub currency_code: String,
    /// sequential, tree, grouped
    pub row_display_order: String,
    pub column_display_order: String,
    /// none, thousands, millions, units
    pub rounding_option: String,
    pub show_zero_amounts: bool,
    pub segment_filter: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Report Row definition (a line on the report)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportRow {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub row_number: i32,
    /// header, data, total, subtotal, separator, text
    pub line_type: String,
    pub label: String,
    pub indent_level: i32,
    /// Account range filter for data rows
    pub account_range_from: Option<String>,
    pub account_range_to: Option<String>,
    pub account_filter: serde_json::Value,
    /// Compute action: total, subtotal, variance, percent, constant
    pub compute_action: Option<String>,
    /// Row IDs to use for computation
    pub compute_source_rows: serde_json::Value,
    pub show_line: bool,
    pub bold: bool,
    pub underline: bool,
    pub double_underline: bool,
    pub page_break_before: bool,
    pub scaling_factor: Option<String>,
    pub parent_row_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Report Column definition (period/scenario columns across the top)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportColumn {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub column_number: i32,
    /// actuals, budget, variance, percent_variance, prior_year, ytd, qtd, custom
    pub column_type: String,
    pub header_label: String,
    pub sub_header_label: Option<String>,
    /// Period offset from the base period (-1 = prior period, 0 = current)
    pub period_offset: i32,
    /// period, qtd, ytd, inception_to_date
    pub period_type: String,
    /// Compute action for calculated columns
    pub compute_action: Option<String>,
    pub compute_source_columns: serde_json::Value,
    pub show_column: bool,
    pub column_width: Option<i32>,
    pub format_override: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Financial Report Run (an execution instance of a template)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub run_number: String,
    pub name: Option<String>,
    pub description: Option<String>,
    /// draft, generated, approved, published, archived
    pub status: String,
    pub as_of_date: Option<chrono::NaiveDate>,
    pub period_from: Option<chrono::NaiveDate>,
    pub period_to: Option<chrono::NaiveDate>,
    pub currency_code: String,
    pub segment_filter: serde_json::Value,
    pub include_unposted: bool,
    pub total_debit: String,
    pub total_credit: String,
    pub net_change: String,
    pub beginning_balance: String,
    pub ending_balance: String,
    pub row_count: i32,
    pub generated_by: Option<Uuid>,
    pub generated_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub published_by: Option<Uuid>,
    pub published_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single cell in a generated report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportResult {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub row_id: Uuid,
    pub column_id: Uuid,
    pub row_number: i32,
    pub column_number: i32,
    pub amount: String,
    pub debit_amount: String,
    pub credit_amount: String,
    pub beginning_balance: String,
    pub ending_balance: String,
    pub is_computed: bool,
    pub compute_note: Option<String>,
    pub display_amount: Option<String>,
    pub display_format: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Favourite report (user bookmark)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportFavourite {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub template_id: Uuid,
    pub display_name: Option<String>,
    pub position: i32,
    pub created_at: DateTime<Utc>,
}

/// Financial Reporting Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportingSummary {
    pub template_count: i32,
    pub active_template_count: i32,
    pub run_count: i32,
    pub recent_runs: Vec<FinancialReportRun>,
    pub templates_by_type: serde_json::Value,
    pub total_amount_reported: String,
}

// ════════════════════════════════════════════════════════════════════════════════
// Withholding Tax Management (Oracle Fusion Payables > Withholding Tax)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Withholding Tax provides:
// - Withholding Tax Codes: Define individual withholding tax types with rates
// - Withholding Tax Groups: Group multiple tax codes into reusable sets
// - Supplier Assignments: Assign withholding tax groups to suppliers
// - Withholding Thresholds: Minimum amounts before withholding applies
// - Automatic Computation: Calculate withholding amounts during payment
// - Withholding Certificates: Track and report withheld taxes
// - Exemptions: Manage supplier exemptions from withholding
//
// Oracle Fusion equivalent: Financials > Payables > Withholding Tax

/// Withholding Tax Code definition
/// Defines an individual withholding tax type with its rate and account.
/// Oracle Fusion equivalent: Payables > Withholding Tax > Tax Codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxCode {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code (e.g., "FEDERAL_WHT", "STATE_WHT", "VAT_WHT")
    pub code: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Tax type: "income_tax", "vat", "service_tax", "contract_tax", "royalty",
    ///           "dividend", "interest", "other"
    pub tax_type: String,
    /// Withholding rate percentage
    pub rate_percentage: String,
    /// Minimum threshold amount below which no withholding applies
    pub threshold_amount: String,
    /// Whether the threshold is cumulative (year-to-date) or per-invoice
    pub threshold_is_cumulative: bool,
    /// GL account code for the withholding liability
    pub withholding_account_code: Option<String>,
    /// GL account code for the withholding expense
    pub expense_account_code: Option<String>,
    /// Whether this tax code is active
    pub is_active: bool,
    /// Effective dates
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update withholding tax code request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxCodeRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_wht_tax_type")]
    pub tax_type: String,
    pub rate_percentage: String,
    #[serde(default = "default_zero_str")]
    pub threshold_amount: String,
    #[serde(default)]
    pub threshold_is_cumulative: bool,
    pub withholding_account_code: Option<String>,
    pub expense_account_code: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_wht_tax_type() -> String { "income_tax".to_string() }
fn default_zero_str() -> String { "0".to_string() }

/// Withholding Tax Group
/// Groups multiple withholding tax codes into a reusable set assignable to suppliers.
/// Oracle Fusion equivalent: Payables > Withholding Tax > Tax Groups
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxGroup {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique group code (e.g., "STD_WHT", "CONTRACTOR_WHT")
    pub code: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Member tax codes
    pub tax_codes: Vec<WithholdingTaxGroupMember>,
    /// Whether this group is active
    pub is_active: bool,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Member of a withholding tax group
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxGroupMember {
    pub id: Uuid,
    pub group_id: Uuid,
    /// Reference to the withholding tax code
    pub tax_code_id: Uuid,
    /// Tax code (denormalized)
    pub tax_code: String,
    /// Tax code name (denormalized)
    pub tax_code_name: String,
    /// Optional rate override percentage (overrides the tax code default)
    pub rate_override: Option<String>,
    /// Whether this member is active in the group
    pub is_active: bool,
    /// Display order
    pub display_order: i32,
}

/// Create withholding tax group request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxGroupRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Tax code IDs to include in the group
    pub tax_code_ids: Vec<Uuid>,
}

/// Supplier Withholding Tax Assignment
/// Links a supplier to a withholding tax group.
/// Oracle Fusion equivalent: Payables > Suppliers > Withholding Tax
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierWithholdingAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Supplier reference
    pub supplier_id: Uuid,
    /// Supplier number (denormalized)
    pub supplier_number: Option<String>,
    /// Supplier name (denormalized)
    pub supplier_name: Option<String>,
    /// Assigned withholding tax group
    pub tax_group_id: Uuid,
    /// Tax group code (denormalized)
    pub tax_group_code: String,
    /// Tax group name (denormalized)
    pub tax_group_name: String,
    /// Whether the supplier is exempt from withholding
    pub is_exempt: bool,
    /// Exemption reason (if exempt)
    pub exemption_reason: Option<String>,
    /// Exemption certificate number
    pub exemption_certificate: Option<String>,
    /// Exemption valid until
    pub exemption_valid_until: Option<chrono::NaiveDate>,
    /// Whether this assignment is active
    pub is_active: bool,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update supplier withholding assignment request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierWithholdingAssignmentRequest {
    pub supplier_id: Uuid,
    pub tax_group_code: String,
    pub is_exempt: Option<bool>,
    pub exemption_reason: Option<String>,
    pub exemption_certificate: Option<String>,
    pub exemption_valid_until: Option<chrono::NaiveDate>,
}

/// Withholding Tax Certificate
/// Certificate issued for tax withheld from a supplier payment.
/// Oracle Fusion equivalent: Payables > Withholding Tax > Certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingCertificate {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Auto-generated certificate number
    pub certificate_number: String,
    /// Supplier information
    pub supplier_id: Uuid,
    pub supplier_number: Option<String>,
    pub supplier_name: Option<String>,
    /// Tax type (from the tax code)
    pub tax_type: String,
    /// Tax code reference
    pub tax_code_id: Uuid,
    pub tax_code: String,
    /// Period the certificate covers
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    /// Amounts
    pub total_invoice_amount: String,
    pub total_withheld_amount: String,
    /// Rate applied
    pub rate_percentage: String,
    /// Payment references covered by this certificate
    pub payment_ids: serde_json::Value,
    /// Status: "draft", "issued", "acknowledged", "cancelled"
    pub status: String,
    /// Date the certificate was issued
    pub issued_at: Option<DateTime<Utc>>,
    /// Date acknowledged by supplier
    pub acknowledged_at: Option<DateTime<Utc>>,
    /// Notes
    pub notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Withholding Tax Line (computed withholding on a payment)
/// Records the actual tax withheld from a specific payment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingTaxLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Payment reference
    pub payment_id: Uuid,
    pub payment_number: Option<String>,
    /// Invoice reference
    pub invoice_id: Uuid,
    pub invoice_number: Option<String>,
    /// Supplier information
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    /// Tax code applied
    pub tax_code_id: Uuid,
    pub tax_code: String,
    pub tax_code_name: Option<String>,
    /// Tax type
    pub tax_type: String,
    /// Rate applied
    pub rate_percentage: String,
    /// Amounts
    pub taxable_amount: String,
    pub withheld_amount: String,
    /// GL account
    pub withholding_account_code: Option<String>,
    /// Status: "pending", "withheld", "remitted", "refunded"
    pub status: String,
    /// Remittance tracking
    pub remittance_date: Option<chrono::NaiveDate>,
    pub remittance_reference: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Result of a withholding tax computation for a payment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingComputationResult {
    /// The supplier's tax group (if assigned)
    pub tax_group_code: Option<String>,
    /// Whether the supplier is exempt
    pub is_exempt: bool,
    /// Individual withholding lines computed
    pub lines: Vec<WithholdingComputedLine>,
    /// Total amount subject to withholding
    pub total_taxable_amount: String,
    /// Total withholding amount
    pub total_withheld_amount: String,
    /// Net payment amount (after withholding)
    pub net_payment_amount: String,
}

/// Single computed withholding line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingComputedLine {
    pub tax_code_id: Uuid,
    pub tax_code: String,
    pub tax_type: String,
    pub rate_percentage: String,
    pub threshold_amount: String,
    pub taxable_amount: String,
    pub withheld_amount: String,
    pub withholding_account_code: Option<String>,
    /// Whether withholding was skipped due to threshold
    pub threshold_applied: bool,
}

/// Withholding Tax Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithholdingSummary {
    /// Total active tax codes
    pub active_tax_code_count: i32,
    /// Total tax groups
    pub tax_group_count: i32,
    /// Total assigned suppliers
    pub assigned_supplier_count: i32,
    /// Total exempt suppliers
    pub exempt_supplier_count: i32,
    /// Total withheld (period)
    pub total_withheld_amount: String,
    /// Total remitted (period)
    pub total_remitted_amount: String,
    /// Total pending remittance
    pub total_pending_remittance: String,
    /// Withholding by tax type
    pub by_tax_type: serde_json::Value,
    /// Withholding by supplier (top suppliers)
    pub by_supplier: serde_json::Value,
    /// Recent certificates
    pub certificates_issued: i32,
}

// ============================================================================
// Multi-Book Accounting (Secondary Ledgers)
// Oracle Fusion equivalent: General Ledger > Multi-Book Accounting
// ============================================================================

/// Accounting Book (Primary or Secondary)
/// Represents a complete accounting representation with its own chart of accounts,
/// calendar, and currency. Primary book is the main ledger; secondary books
/// represent alternate accounting standards (e.g., IFRS, local GAAP, statutory).
/// Oracle Fusion equivalent: General Ledger > Accounting Books
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingBook {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique book code (e.g., "PRIMARY_GAAP", "IFRS_BOOK", "LOCAL_STATUTORY")
    pub code: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Book type: "primary" or "secondary"
    pub book_type: String,
    /// Chart of accounts identifier / code
    pub chart_of_accounts_code: String,
    /// Accounting calendar code (references period-close calendars)
    pub calendar_code: String,
    /// Base currency code for this book
    pub currency_code: String,
    /// Whether this book is enabled for posting
    pub is_enabled: bool,
    /// Whether auto-propagation from primary is enabled (secondary books only)
    pub auto_propagation_enabled: bool,
    /// Mapping level: "journal" or "subledger"
    pub mapping_level: String,
    /// Status: "draft", "active", "inactive", "suspended"
    pub status: String,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update accounting book request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountingBookRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub book_type: String,
    pub chart_of_accounts_code: String,
    pub calendar_code: String,
    pub currency_code: String,
    pub auto_propagation_enabled: Option<bool>,
    pub mapping_level: Option<String>,
}

/// Account Mapping Rule
/// Maps account segments from a source book to a target book.
/// Oracle Fusion equivalent: General Ledger > Multi-Book > Account Mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountMapping {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Source accounting book ID
    pub source_book_id: Uuid,
    /// Target accounting book ID
    pub target_book_id: Uuid,
    /// Source account code / range
    pub source_account_code: String,
    /// Target account code
    pub target_account_code: String,
    /// Optional segment-level mappings (JSON: {"segment_name": "value"})
    pub segment_mappings: serde_json::Value,
    /// Priority (lower = higher priority)
    pub priority: i32,
    /// Whether this rule is active
    pub is_active: bool,
    /// Effective dates
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    /// Audit
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create account mapping request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountMappingRequest {
    pub source_book_id: Uuid,
    pub target_book_id: Uuid,
    pub source_account_code: String,
    pub target_account_code: String,
    pub segment_mappings: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Book Journal Entry
/// A journal entry in a specific accounting book, either posted directly
/// or propagated from another book.
/// Oracle Fusion equivalent: General Ledger > Multi-Book > Journal Entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookJournalEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// The accounting book this entry belongs to
    pub book_id: Uuid,
    /// Journal entry number (auto-generated within the book)
    pub entry_number: String,
    /// Journal header description
    pub header_description: Option<String>,
    /// Source book ID (if propagated)
    pub source_book_id: Option<Uuid>,
    /// Source journal entry ID (if propagated)
    pub source_entry_id: Option<Uuid>,
    /// External reference (e.g., subledger transaction ID)
    pub external_reference: Option<String>,
    /// Accounting date
    pub accounting_date: chrono::NaiveDate,
    /// Period name
    pub period_name: Option<String>,
    /// Total debit amount
    pub total_debit: String,
    /// Total credit amount
    pub total_credit: String,
    /// Status: "draft", "posted", "propagated", "reversed"
    pub status: String,
    /// Whether this was auto-propagated
    pub is_auto_propagated: bool,
    /// Currency
    pub currency_code: String,
    /// Conversion rate (if different from source book currency)
    pub conversion_rate: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    /// Audit
    pub created_by: Option<Uuid>,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Book Journal Line
/// Individual debit/credit line within a book journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookJournalLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent journal entry
    pub entry_id: Uuid,
    /// Line number within the entry
    pub line_number: i32,
    /// Account code in this book's chart of accounts
    pub account_code: String,
    /// Account name (denormalized)
    pub account_name: Option<String>,
    /// Debit amount
    pub debit_amount: String,
    /// Credit amount
    pub credit_amount: String,
    /// Description
    pub description: Option<String>,
    /// Tax code
    pub tax_code: Option<String>,
    /// Source line ID (if propagated)
    pub source_line_id: Option<Uuid>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create book journal entry request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookJournalEntryRequest {
    pub book_id: Uuid,
    pub header_description: Option<String>,
    pub external_reference: Option<String>,
    pub accounting_date: chrono::NaiveDate,
    pub period_name: Option<String>,
    pub currency_code: String,
    pub lines: Vec<BookJournalLineRequest>,
}

/// Create book journal line request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookJournalLineRequest {
    pub account_code: String,
    pub account_name: Option<String>,
    pub debit_amount: String,
    pub credit_amount: String,
    pub description: Option<String>,
    pub tax_code: Option<String>,
}

/// Propagation Log Entry
/// Tracks the propagation of journal entries between books.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropagationLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Source book
    pub source_book_id: Uuid,
    /// Target book
    pub target_book_id: Uuid,
    /// Source journal entry
    pub source_entry_id: Uuid,
    /// Created target journal entry
    pub target_entry_id: Option<Uuid>,
    /// Status: "pending", "completed", "failed", "skipped"
    pub status: String,
    /// Number of lines propagated
    pub lines_propagated: i32,
    /// Number of lines unmapped (skipped)
    pub lines_unmapped: i32,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Propagation timestamp
    pub propagated_at: DateTime<Utc>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Multi-Book Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiBookSummary {
    /// Total accounting books
    pub book_count: i32,
    /// Primary book code
    pub primary_book_code: Option<String>,
    /// Count of secondary books
    pub secondary_book_count: i32,
    /// Active mapping rules count
    pub mapping_rule_count: i32,
    /// Recent propagations count
    pub recent_propagation_count: i32,
    /// Propagation success rate
    pub propagation_success_rate: String,
    /// Unposted entries by book
    pub unposted_entries_by_book: serde_json::Value,
    /// Journal entry counts by book
    pub entry_counts_by_book: serde_json::Value,
}

// ============================================================================
// Procurement Contracts (Oracle Fusion SCM > Procurement > Contracts)
// ============================================================================

/// Contract type definition (e.g. Blanket Purchase Agreement, Contract Purchase Agreement)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractType {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code for the contract type
    pub code: String,
    /// Display name
    pub name: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Type classification: "blanket", "purchase_agreement", "service", "lease", "other"
    pub contract_classification: String,
    /// Whether this contract type requires approval before activation
    pub requires_approval: bool,
    /// Default contract duration in days (optional)
    pub default_duration_days: Option<i32>,
    /// Whether the contract allows amount-based commitments
    pub allow_amount_commitment: bool,
    /// Whether the contract allows quantity-based commitments
    pub allow_quantity_commitment: bool,
    /// Whether contract lines can be added after activation
    pub allow_line_additions: bool,
    /// Whether contract price can be adjusted
    pub allow_price_adjustment: bool,
    /// Whether renewal is allowed
    pub allow_renewal: bool,
    /// Whether termination is allowed
    pub allow_termination: bool,
    /// Maximum number of renewals (None = unlimited)
    pub max_renewals: Option<i32>,
    /// Default payment terms code
    pub default_payment_terms_code: Option<String>,
    /// Default currency code
    pub default_currency_code: Option<String>,
    /// Whether this contract type is active
    pub is_active: bool,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Procurement contract header
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcurementContract {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// System-generated contract number
    pub contract_number: String,
    /// Descriptive title
    pub title: String,
    /// Description of the contract
    pub description: Option<String>,
    /// Contract type code
    pub contract_type_code: Option<String>,
    /// Contract classification: "blanket", "purchase_agreement", "service", "lease", "other"
    pub contract_classification: String,
    /// Current status: "draft", "pending_approval", "active", "expired", "terminated", "closed"
    pub status: String,
    /// Supplier/vendor UUID
    pub supplier_id: Uuid,
    /// Supplier number
    pub supplier_number: Option<String>,
    /// Supplier name
    pub supplier_name: Option<String>,
    /// Supplier contact name
    pub supplier_contact: Option<String>,
    /// Buyer/procurement officer UUID
    pub buyer_id: Option<Uuid>,
    /// Buyer name
    pub buyer_name: Option<String>,
    /// Contract start date
    pub start_date: Option<chrono::NaiveDate>,
    /// Contract end date
    pub end_date: Option<chrono::NaiveDate>,
    /// Total committed amount
    pub total_committed_amount: String,
    /// Total released (ordered) amount against this contract
    pub total_released_amount: String,
    /// Total invoiced amount against this contract
    pub total_invoiced_amount: String,
    /// Currency code
    pub currency_code: String,
    /// Payment terms code
    pub payment_terms_code: Option<String>,
    /// Whether price is fixed or variable
    pub price_type: String,
    /// Number of renewals so far
    pub renewal_count: i32,
    /// Maximum allowed renewals
    pub max_renewals: Option<i32>,
    /// Number of contract lines
    pub line_count: i32,
    /// Number of milestones
    pub milestone_count: i32,
    /// Approver UUID
    pub approved_by: Option<Uuid>,
    /// Approval timestamp
    pub approved_at: Option<DateTime<Utc>>,
    /// Rejection reason (if applicable)
    pub rejection_reason: Option<String>,
    /// Termination reason (if applicable)
    pub termination_reason: Option<String>,
    /// Terminated by
    pub terminated_by: Option<Uuid>,
    /// Termination date
    pub terminated_at: Option<DateTime<Utc>>,
    /// Notes
    pub notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Procurement contract line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent contract
    pub contract_id: Uuid,
    /// Line number within contract
    pub line_number: i32,
    /// Product/item description
    pub item_description: String,
    /// Product/item code or SKU
    pub item_code: Option<String>,
    /// Product category
    pub category: Option<String>,
    /// Unit of measure
    pub uom: Option<String>,
    /// Quantity committed
    pub quantity_committed: Option<String>,
    /// Quantity released (ordered so far)
    pub quantity_released: String,
    /// Unit price
    pub unit_price: String,
    /// Line amount (quantity_committed * unit_price)
    pub line_amount: String,
    /// Amount released (invoiced/spent so far)
    pub amount_released: String,
    /// Delivery date
    pub delivery_date: Option<chrono::NaiveDate>,
    /// Supplier part number
    pub supplier_part_number: Option<String>,
    /// GL account code
    pub account_code: Option<String>,
    /// Cost center
    pub cost_center: Option<String>,
    /// Project ID
    pub project_id: Option<Uuid>,
    /// Line notes
    pub notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Contract milestone / deliverable
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractMilestone {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent contract
    pub contract_id: Uuid,
    /// Optional parent line
    pub contract_line_id: Option<Uuid>,
    /// Milestone sequence number
    pub milestone_number: i32,
    /// Milestone name/title
    pub name: String,
    /// Detailed description
    pub description: Option<String>,
    /// Milestone type: "delivery", "payment", "review", "acceptance", "custom"
    pub milestone_type: String,
    /// Target completion date
    pub target_date: chrono::NaiveDate,
    /// Actual completion date
    pub actual_date: Option<chrono::NaiveDate>,
    /// Status: "pending", "in_progress", "completed", "overdue", "cancelled"
    pub status: String,
    /// Amount associated with this milestone
    pub amount: String,
    /// Percentage of total contract (for progress tracking)
    pub percent_of_total: String,
    /// Deliverable description
    pub deliverable: Option<String>,
    /// Whether this milestone is billable
    pub is_billable: bool,
    /// Approved by
    pub approved_by: Option<Uuid>,
    /// Approved date
    pub approved_at: Option<DateTime<Utc>>,
    /// Notes
    pub notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Contract renewal record
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractRenewal {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent contract
    pub contract_id: Uuid,
    /// Renewal number (1 for first renewal, etc.)
    pub renewal_number: i32,
    /// Previous end date
    pub previous_end_date: chrono::NaiveDate,
    /// New end date
    pub new_end_date: chrono::NaiveDate,
    /// Renewal type: "automatic", "manual", "negotiated"
    pub renewal_type: String,
    /// Any terms that changed during renewal
    pub terms_changed: Option<String>,
    /// Renewed by
    pub renewed_by: Option<Uuid>,
    /// Renewal date
    pub renewed_at: DateTime<Utc>,
    /// Notes
    pub notes: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Contract spend entry (tracks actual spend against contract)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractSpend {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Parent contract
    pub contract_id: Uuid,
    /// Optional parent contract line
    pub contract_line_id: Option<Uuid>,
    /// Source document type (e.g. "purchase_order", "invoice")
    pub source_type: String,
    /// Source document ID
    pub source_id: Option<Uuid>,
    /// Source document number
    pub source_number: Option<String>,
    /// Transaction date
    pub transaction_date: chrono::NaiveDate,
    /// Amount
    pub amount: String,
    /// Quantity (if applicable)
    pub quantity: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Procurement Contracts dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractDashboardSummary {
    /// Total contracts count
    pub total_contracts: i32,
    /// Active contracts count
    pub active_contracts: i32,
    /// Contracts expiring within 30 days
    pub expiring_contracts_count: i32,
    /// Total committed amount across all active contracts
    pub total_committed_amount: String,
    /// Total released/spent amount
    pub total_released_amount: String,
    /// Utilization percentage (released / committed)
    pub utilization_percent: String,
    /// Contracts by status
    pub contracts_by_status: serde_json::Value,
    /// Contracts by type
    pub contracts_by_type: serde_json::Value,
    /// Top suppliers by committed amount
    pub top_suppliers: serde_json::Value,
}

// ════════════════════════════════════════════════════════════════════════════════
// Inventory Management (Oracle Fusion SCM > Inventory Management)
// ════════════════════════════════════════════════════════════════════════════════
//
// Oracle Fusion Cloud ERP Inventory Management provides:
// - Inventory Organizations: Warehouses, stores, and distribution centers
// - Items: Products, materials, and supplies with full attribute tracking
// - Item Categories: Hierarchical classification of items
// - Subinventories: Logical storage areas within organizations
// - Locators: Specific bins/shelves within subinventories
// - On-Hand Balances: Real-time stock quantities with lot/serial/revision tracking
// - Inventory Transactions: All material movements (receipts, issues, transfers, adjustments)
// - Transaction Types: Configurable transaction type definitions
// - Cycle Counts: Periodic stock verification with variance analysis
// - Transaction Reasons: Coded reasons for material movements
//
// Oracle Fusion equivalent: SCM > Inventory Management

/// Inventory Organization (warehouse, store, distribution center)
/// Oracle Fusion: Inventory > Organizations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryOrganization {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// "warehouse", "store", "distribution_center", "manufacturing", "other"
    pub org_type: String,
    pub location_code: Option<String>,
    pub address: Option<serde_json::Value>,
    pub is_active: bool,
    pub default_subinventory_code: Option<String>,
    pub default_currency_code: String,
    pub requires_approval_for_issues: bool,
    pub requires_approval_for_transfers: bool,
    pub enable_lot_control: bool,
    pub enable_serial_control: bool,
    pub enable_revision_control: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Item Category (hierarchical)
/// Oracle Fusion: Inventory > Item Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub track_as_asset: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Item (product, material, supply)
/// Oracle Fusion: Inventory > Items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_code: String,
    pub name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_code: Option<String>,
    /// "inventory", "non_inventory", "service", "expense", "capital"
    pub item_type: String,
    pub uom: String,
    pub secondary_uom: Option<String>,
    pub weight: Option<String>,
    pub weight_uom: Option<String>,
    pub volume: Option<String>,
    pub volume_uom: Option<String>,
    pub list_price: String,
    pub standard_cost: String,
    pub min_order_quantity: Option<String>,
    pub max_order_quantity: Option<String>,
    pub lead_time_days: i32,
    pub shelf_life_days: Option<i32>,
    pub is_lot_controlled: bool,
    pub is_serial_controlled: bool,
    pub is_revision_controlled: bool,
    pub is_perishable: bool,
    pub is_hazardous: bool,
    pub is_purchasable: bool,
    pub is_sellable: bool,
    pub is_stockable: bool,
    pub inventory_asset_account_code: Option<String>,
    pub expense_account_code: Option<String>,
    pub cost_of_goods_sold_account: Option<String>,
    pub revenue_account_code: Option<String>,
    pub image_url: Option<String>,
    pub barcode: Option<String>,
    pub supplier_item_codes: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subinventory (logical storage area)
/// Oracle Fusion: Inventory > Subinventories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subinventory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inventory_org_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// "storage", "receiving", "staging", "inspection", "packing", "other"
    pub subinventory_type: String,
    pub asset_subinventory: bool,
    pub quantity_tracked: bool,
    pub location_code: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Locator (bin/shelf/row within a subinventory)
/// Oracle Fusion: Inventory > Locators
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Locator {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subinventory_id: Uuid,
    pub code: String,
    pub description: Option<String>,
    pub picker_order: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// On-Hand Balance (real-time stock quantity)
/// Oracle Fusion: Inventory > On-hand Quantities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnHandBalance {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inventory_org_id: Uuid,
    pub item_id: Uuid,
    pub subinventory_id: Uuid,
    pub locator_id: Option<Uuid>,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub revision: Option<String>,
    pub quantity: String,
    pub reserved_quantity: String,
    pub available_quantity: String,
    pub unit_cost: String,
    pub total_value: String,
    pub last_transaction_date: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inventory Transaction Type
/// Oracle Fusion: Inventory > Transaction Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryTransactionType {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// "receive", "issue", "transfer", "adjustment", "return_to_vendor",
    /// "return_to_customer", "cycle_count_adjustment", "misc_receipt", "misc_issue"
    pub transaction_action: String,
    /// "manual", "purchase_order", "sales_order", "work_order", "system"
    pub source_type: String,
    pub is_system: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inventory Transaction (material movement)
/// Oracle Fusion: Inventory > Transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transaction_number: String,
    pub transaction_type_id: Option<Uuid>,
    pub transaction_type_code: Option<String>,
    pub transaction_action: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub source_line_id: Option<Uuid>,
    pub item_id: Uuid,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    // From location
    pub from_inventory_org_id: Option<Uuid>,
    pub from_subinventory_id: Option<Uuid>,
    pub from_locator_id: Option<Uuid>,
    // To location
    pub to_inventory_org_id: Option<Uuid>,
    pub to_subinventory_id: Option<Uuid>,
    pub to_locator_id: Option<Uuid>,
    // Quantities
    pub quantity: String,
    pub uom: String,
    pub unit_cost: String,
    pub total_cost: String,
    // Lot/Serial/Revision
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub revision: Option<String>,
    // Dates
    pub transaction_date: DateTime<Utc>,
    pub accounting_date: Option<chrono::NaiveDate>,
    // Reference
    pub reason_id: Option<Uuid>,
    pub reason_name: Option<String>,
    pub reference: Option<String>,
    pub reference_type: Option<String>,
    pub notes: Option<String>,
    // GL
    pub is_posted: bool,
    pub posted_at: Option<DateTime<Utc>>,
    pub journal_entry_id: Option<Uuid>,
    // Workflow
    /// "pending", "approved", "processed", "cancelled"
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    // Audit
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cycle Count Header
/// Oracle Fusion: Inventory > Cycle Counts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleCountHeader {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub count_number: String,
    pub name: String,
    pub description: Option<String>,
    pub inventory_org_id: Uuid,
    pub subinventory_id: Option<Uuid>,
    pub count_date: chrono::NaiveDate,
    /// "draft", "in_progress", "completed", "cancelled"
    pub status: String,
    /// "full", "abc", "random", "by_category"
    pub count_method: String,
    pub tolerance_percent: String,
    pub total_items: i32,
    pub counted_items: i32,
    pub matched_items: i32,
    pub mismatched_items: i32,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Cycle Count Line
/// Oracle Fusion: Inventory > Cycle Count Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleCountLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cycle_count_id: Uuid,
    pub line_number: i32,
    pub item_id: Uuid,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub subinventory_id: Option<Uuid>,
    pub locator_id: Option<Uuid>,
    pub lot_number: Option<String>,
    pub revision: Option<String>,
    pub system_quantity: String,
    pub count_quantity_1: Option<String>,
    pub count_quantity_2: Option<String>,
    pub count_quantity_3: Option<String>,
    pub count_date_1: Option<DateTime<Utc>>,
    pub count_date_2: Option<DateTime<Utc>>,
    pub count_date_3: Option<DateTime<Utc>>,
    pub counted_by_1: Option<Uuid>,
    pub counted_by_2: Option<Uuid>,
    pub counted_by_3: Option<Uuid>,
    pub approved_quantity: Option<String>,
    pub variance_quantity: Option<String>,
    pub variance_percent: Option<String>,
    pub is_matched: bool,
    /// "pending", "counted", "recount", "approved", "adjusted"
    pub status: String,
    pub adjustment_transaction_id: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Transaction Reason
/// Oracle Fusion: Inventory > Transaction Reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReason {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub applicable_actions: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inventory Dashboard Summary
/// Oracle Fusion: Inventory Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryDashboardSummary {
    pub total_items: i32,
    pub active_items: i32,
    pub total_organizations: i32,
    pub total_on_hand_value: String,
    pub total_pending_transactions: i32,
    pub total_processed_transactions: i32,
    pub items_by_type: serde_json::Value,
    pub items_by_category: serde_json::Value,
    pub transactions_by_action: serde_json::Value,
    pub top_items_by_value: serde_json::Value,
    pub pending_cycle_counts: i32,
    pub low_stock_items: i32,
}

// ============================================================================
// Customer Returns Management / Return Material Authorization (RMA)
// Oracle Fusion Cloud ERP: Order Management > Returns
// ============================================================================

/// Return reason code definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnReason {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub return_type: String, // standard_return, exchange, repair, warranty
    pub default_disposition: Option<String>, // return_to_stock, scrap, inspect, repair
    pub requires_approval: bool,
    pub credit_issued_automatically: bool,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Return Material Authorization (RMA) header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnAuthorization {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rma_number: String,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub return_type: String, // standard_return, exchange, repair, warranty
    pub status: String, // draft, submitted, approved, rejected, partially_received, received, closed, cancelled
    pub reason_code: Option<String>,
    pub reason_name: Option<String>,
    pub original_order_number: Option<String>,
    pub original_order_id: Option<Uuid>,
    pub customer_contact: Option<String>,
    pub customer_email: Option<String>,
    pub customer_phone: Option<String>,
    pub return_date: chrono::NaiveDate,
    pub expected_receipt_date: Option<chrono::NaiveDate>,
    pub total_quantity: String,
    pub total_amount: String,
    pub total_credit_amount: String,
    pub currency_code: String,
    pub notes: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub credit_memo_id: Option<Uuid>,
    pub credit_memo_number: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// RMA line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rma_id: Uuid,
    pub line_number: i32,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub original_line_id: Option<Uuid>,
    pub original_quantity: String,
    pub return_quantity: String,
    pub unit_price: String,
    pub return_amount: String,
    pub credit_amount: String,
    pub reason_code: Option<String>,
    pub disposition: Option<String>, // return_to_stock, scrap, inspect, repair, exchange
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub condition: Option<String>, // good, damaged, defective, wrong_item
    pub received_quantity: String,
    pub received_date: Option<chrono::NaiveDate>,
    pub inspection_status: Option<String>, // pending, passed, failed, pending_review
    pub inspection_notes: Option<String>,
    pub credit_status: Option<String>, // pending, issued, reversed
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit memo generated from returns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditMemo {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub credit_memo_number: String,
    pub rma_id: Option<Uuid>,
    pub rma_number: Option<String>,
    pub customer_id: Uuid,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub status: String, // draft, issued, applied, partially_applied, reversed, cancelled
    pub applied_amount: String,
    pub remaining_amount: String,
    pub issue_date: Option<chrono::NaiveDate>,
    pub applied_to_invoice_id: Option<Uuid>,
    pub applied_to_invoice_number: Option<String>,
    pub gl_account_code: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Customer Returns dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnsDashboardSummary {
    pub total_rmas: i32,
    pub open_rmas: i32,
    pub pending_approval: i32,
    pub pending_receipt: i32,
    pub pending_inspection: i32,
    pub total_credit_issued_amount: String,
    pub total_credit_pending_amount: String,
    pub rmas_by_status: serde_json::Value,
    pub rmas_by_reason: serde_json::Value,
    pub rmas_by_disposition: serde_json::Value,
    pub top_returned_items: serde_json::Value,
    pub average_processing_days: String,
}

// ============================================================================
// Advanced Pricing Management (Oracle Fusion Order Management > Pricing)
// ============================================================================

/// Price List definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceList {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub currency_code: String,
    pub list_type: String,
    pub pricing_basis: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Price List Line (item-level pricing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceListLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub price_list_id: Uuid,
    pub line_number: i32,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub pricing_unit_of_measure: String,
    pub list_price: String,
    pub unit_price: String,
    pub cost_price: String,
    pub margin_percent: String,
    pub minimum_quantity: String,
    pub maximum_quantity: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Price Tier (quantity break)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceTier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub price_list_line_id: Uuid,
    pub tier_number: i32,
    pub from_quantity: String,
    pub to_quantity: Option<String>,
    pub price: String,
    pub discount_percent: String,
    pub price_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Discount Rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: String,
    pub application_method: String,
    pub stacking_rule: String,
    pub priority: i32,
    pub condition: serde_json::Value,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub is_active: bool,
    pub usage_count: i32,
    pub max_usage: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Charge Definition (shipping, handling, surcharges)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargeDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub charge_type: String,
    pub charge_category: String,
    pub calculation_method: String,
    pub charge_amount: String,
    pub charge_percent: String,
    pub minimum_charge: String,
    pub maximum_charge: Option<String>,
    pub taxable: bool,
    pub condition: serde_json::Value,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pricing Strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingStrategy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub strategy_type: String,
    pub priority: i32,
    pub condition: serde_json::Value,
    pub price_list_id: Option<Uuid>,
    pub markup_percent: String,
    pub markdown_percent: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Price Calculation Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalculationLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub calculation_date: DateTime<Utc>,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub line_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub requested_quantity: Option<String>,
    pub unit_list_price: String,
    pub unit_selling_price: String,
    pub discount_amount: String,
    pub discount_rule_id: Option<Uuid>,
    pub charge_amount: String,
    pub charge_definition_id: Option<Uuid>,
    pub strategy_id: Option<Uuid>,
    pub price_list_id: Option<Uuid>,
    pub calculation_steps: serde_json::Value,
    pub currency_code: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Result of a price calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalculationResult {
    pub list_price: String,
    pub discount_amount: String,
    pub charge_amount: String,
    pub unit_selling_price: String,
    pub extended_price: String,
    pub currency_code: String,
    pub applied_discount_rule_code: Option<String>,
    pub applied_charge_code: Option<String>,
    pub applied_price_list_code: Option<String>,
    pub applied_strategy_code: Option<String>,
    pub tier_applied: Option<i32>,
    pub calculation_steps: Vec<PriceCalculationStep>,
}

/// A single step in the pricing calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCalculationStep {
    pub step_type: String,
    pub description: String,
    pub amount_before: String,
    pub amount_after: String,
    pub rule_applied: Option<String>,
}

/// Pricing dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingDashboardSummary {
    pub total_price_lists: i32,
    pub active_price_lists: i32,
    pub total_discount_rules: i32,
    pub active_discount_rules: i32,
    pub total_charge_definitions: i32,
    pub total_strategies: i32,
    pub total_calculations_today: i32,
    pub price_lists_by_status: serde_json::Value,
    pub discount_rules_by_type: serde_json::Value,
    pub charges_by_type: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════
// Sales Commission Management
// Oracle Fusion Cloud ERP: Incentive Compensation
// ═══════════════════════════════════════════════════════════════════

/// Sales Representative profile for commission tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesRepresentative {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rep_code: String,
    pub employee_id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub territory_code: Option<String>,
    pub territory_name: Option<String>,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
    pub hire_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Commission Plan defining how a rep earns commission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub basis: String,
    pub calculation_method: String,
    pub default_rate: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Commission rate tier within a plan (e.g., 0-10k at 5%, 10k-50k at 8%)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionRateTier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Uuid,
    pub tier_number: i32,
    pub from_amount: String,
    pub to_amount: Option<String>,
    pub rate_percent: String,
    pub flat_amount: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Plan assignment linking a rep to a commission plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rep_id: Uuid,
    pub plan_id: Uuid,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Sales Quota for a rep in a given period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesQuota {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rep_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub quota_number: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub quota_type: String,
    pub target_amount: String,
    pub achieved_amount: String,
    pub achievement_percent: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Commission transaction (a credited sale earning commission)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rep_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub quota_id: Option<Uuid>,
    pub transaction_number: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub sale_amount: String,
    pub commission_basis_amount: String,
    pub commission_rate: String,
    pub commission_amount: String,
    pub currency_code: String,
    pub status: String,
    pub payout_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Commission payout batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionPayout {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payout_number: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub total_payout_amount: String,
    pub currency_code: String,
    pub rep_count: i32,
    pub transaction_count: i32,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rejected_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Individual payout line per rep within a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionPayoutLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payout_id: Uuid,
    pub rep_id: Uuid,
    pub rep_name: String,
    pub plan_id: Option<Uuid>,
    pub plan_code: Option<String>,
    pub gross_commission: String,
    pub adjustment_amount: String,
    pub net_commission: String,
    pub currency_code: String,
    pub transaction_count: i32,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Commission Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionDashboardSummary {
    pub total_reps: i32,
    pub active_reps: i32,
    pub total_plans: i32,
    pub active_plans: i32,
    pub total_quotas: i32,
    pub total_transactions: i32,
    pub total_pending_payouts: i32,
    pub total_commission_this_month: String,
    pub total_quota_achievement_percent: String,
    pub payouts_by_status: serde_json::Value,
    pub top_performers: Vec<CommissionTopPerformer>,
}

/// Top performer in commission dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionTopPerformer {
    pub rep_id: Uuid,
    pub rep_name: String,
    pub total_commission: String,
    pub quota_achievement: String,
    pub rank: i32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Treasury Management (Oracle Fusion Treasury)
// ═══════════════════════════════════════════════════════════════════════════════

/// Counterparty (bank or financial institution) for treasury operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryCounterparty {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub counterparty_code: String,
    pub name: String,
    pub counterparty_type: String, // bank, financial_institution, internal
    pub country_code: Option<String>,
    pub credit_rating: Option<String>,
    pub credit_limit: Option<String>,
    pub settlement_currency: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Treasury deal (investment, borrowing, or FX deal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryDeal {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub deal_number: String,
    pub deal_type: String, // investment, borrowing, fx_forward, fx_spot
    pub description: Option<String>,
    pub counterparty_id: Uuid,
    pub counterparty_name: Option<String>,
    pub currency_code: String,
    pub principal_amount: String,
    pub interest_rate: Option<String>,
    pub interest_basis: Option<String>, // actual_360, actual_365, 30_360
    pub start_date: chrono::NaiveDate,
    pub maturity_date: chrono::NaiveDate,
    pub term_days: i32,
    /// For FX deals: the bought currency
    pub fx_buy_currency: Option<String>,
    /// For FX deals: the bought amount
    pub fx_buy_amount: Option<String>,
    /// For FX deals: the sold currency
    pub fx_sell_currency: Option<String>,
    /// For FX deals: the sold amount
    pub fx_sell_amount: Option<String>,
    /// For FX deals: the exchange rate
    pub fx_rate: Option<String>,
    pub accrued_interest: String,
    pub settlement_amount: Option<String>,
    pub gl_account_code: Option<String>,
    pub status: String, // draft, authorized, settled, matured, cancelled
    pub authorized_by: Option<Uuid>,
    pub authorized_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
    pub matured_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Treasury deal settlement record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasurySettlement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub deal_id: Uuid,
    pub settlement_number: String,
    pub settlement_type: String, // full, partial, early
    pub settlement_date: chrono::NaiveDate,
    pub principal_amount: String,
    pub interest_amount: String,
    pub total_amount: String,
    pub payment_reference: Option<String>,
    pub journal_entry_id: Option<Uuid>,
    pub status: String, // pending, completed, reversed
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Treasury dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreasuryDashboardSummary {
    pub total_active_deals: i32,
    pub total_investments: String,
    pub total_borrowings: String,
    pub total_fx_exposure: String,
    pub total_accrued_interest: String,
    pub deals_maturing_7_days: i32,
    pub deals_maturing_30_days: i32,
    pub investment_count: i32,
    pub borrowing_count: i32,
    pub fx_deal_count: i32,
    pub active_counterparties: i32,
    pub deals_by_status: serde_json::Value,
    pub deals_by_type: serde_json::Value,
    pub maturity_profile: serde_json::Value,
}

// ============================================================================
// Subscription Management (Oracle Fusion Subscription Management)
// ============================================================================

/// Subscription product in the catalog
/// Oracle Fusion: Subscription Management > Products
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionProduct {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub product_code: String,
    pub name: String,
    pub description: Option<String>,
    pub product_type: String,
    pub billing_frequency: String,
    pub default_duration_months: i32,
    pub is_auto_renew: bool,
    pub cancellation_notice_days: i32,
    pub setup_fee: String,
    pub tier_type: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription product price tier
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionPriceTier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub product_id: Uuid,
    pub tier_name: Option<String>,
    pub min_quantity: String,
    pub max_quantity: Option<String>,
    pub unit_price: String,
    pub discount_percent: String,
    pub currency_code: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription (main header)
/// Oracle Fusion: Subscription Management > Subscriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subscription_number: String,
    pub customer_id: Uuid,
    pub customer_name: Option<String>,
    pub product_id: Uuid,
    pub product_code: Option<String>,
    pub product_name: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub renewal_date: Option<chrono::NaiveDate>,
    pub billing_frequency: String,
    pub billing_day_of_month: i32,
    pub billing_alignment: String,
    pub currency_code: String,
    pub quantity: String,
    pub unit_price: String,
    pub list_price: String,
    pub discount_percent: String,
    pub setup_fee: String,
    pub recurring_amount: String,
    pub total_contract_value: String,
    pub total_billed: String,
    pub total_revenue_recognized: String,
    pub duration_months: i32,
    pub is_auto_renew: bool,
    pub cancellation_date: Option<chrono::NaiveDate>,
    pub cancellation_reason: Option<String>,
    pub suspension_reason: Option<String>,
    pub sales_rep_id: Option<Uuid>,
    pub sales_rep_name: Option<String>,
    pub gl_revenue_account: Option<String>,
    pub gl_deferred_account: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription amendment (change to an active subscription)
/// Oracle Fusion: Subscription Management > Amendments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionAmendment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subscription_id: Uuid,
    pub amendment_number: String,
    pub amendment_type: String,
    pub description: Option<String>,
    pub old_quantity: Option<String>,
    pub new_quantity: Option<String>,
    pub old_unit_price: Option<String>,
    pub new_unit_price: Option<String>,
    pub old_recurring_amount: Option<String>,
    pub new_recurring_amount: Option<String>,
    pub old_end_date: Option<chrono::NaiveDate>,
    pub new_end_date: Option<chrono::NaiveDate>,
    pub effective_date: chrono::NaiveDate,
    pub proration_credit: Option<String>,
    pub proration_charge: Option<String>,
    pub status: String,
    pub applied_at: Option<DateTime<Utc>>,
    pub applied_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription billing schedule line
/// Oracle Fusion: Subscription Management > Billing Schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionBillingLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subscription_id: Uuid,
    pub schedule_number: i32,
    pub billing_date: chrono::NaiveDate,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub amount: String,
    pub proration_amount: String,
    pub total_amount: String,
    pub invoice_id: Option<Uuid>,
    pub invoice_number: Option<String>,
    pub status: String,
    pub paid_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription revenue schedule line (ASC 606 / IFRS 15)
/// Oracle Fusion: Subscription Management > Revenue Schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionRevenueLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subscription_id: Uuid,
    pub billing_schedule_id: Option<Uuid>,
    pub period_name: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub revenue_amount: String,
    pub deferred_amount: String,
    pub recognized_to_date: String,
    pub status: String,
    pub recognized_at: Option<DateTime<Utc>>,
    pub journal_entry_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Subscription dashboard summary
/// Oracle Fusion: Subscription Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionDashboardSummary {
    pub total_active_subscriptions: i32,
    pub total_subscribers: i32,
    pub total_monthly_recurring_revenue: String,
    pub total_annual_recurring_revenue: String,
    pub total_contract_value: String,
    pub total_billed: String,
    pub total_revenue_recognized: String,
    pub total_deferred_revenue: String,
    pub churn_rate_percent: String,
    pub renewals_due_30_days: i32,
    pub new_subscriptions_this_month: i32,
    pub cancelled_this_month: i32,
    pub subscriptions_by_status: serde_json::Value,
    pub revenue_by_product: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════
// Grant Management (Oracle Fusion Grants Management)
// ═══════════════════════════════════════════════════════════════

/// Grant Sponsor (funding organization)
/// Oracle Fusion: Grants Management > Sponsors
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantSponsor {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub sponsor_code: String,
    pub name: String,
    pub sponsor_type: String,
    pub country_code: Option<String>,
    pub taxpayer_id: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub payment_terms: Option<String>,
    pub billing_frequency: String,
    pub currency_code: String,
    pub credit_limit: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Indirect Cost Rate Agreement
/// Oracle Fusion: Grants Management > Indirect Cost Rates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantIndirectCostRate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rate_name: String,
    pub rate_type: String,
    pub rate_percentage: String,
    pub base_type: String,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub negotiated_by: Option<String>,
    pub approved_by: Option<Uuid>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Grant Award
/// Oracle Fusion: Grants Management > Awards
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantAward {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub award_number: String,
    pub award_title: String,
    pub sponsor_id: Uuid,
    pub sponsor_name: Option<String>,
    pub sponsor_award_number: Option<String>,
    pub status: String,
    pub award_type: String,
    pub award_purpose: Option<String>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub budget_start_date: Option<chrono::NaiveDate>,
    pub budget_end_date: Option<chrono::NaiveDate>,
    pub total_award_amount: String,
    pub direct_costs_total: String,
    pub indirect_costs_total: String,
    pub cost_sharing_total: String,
    pub total_funded: String,
    pub total_billed: String,
    pub total_collected: String,
    pub total_expenditures: String,
    pub total_commitments: String,
    pub available_balance: String,
    pub currency_code: String,
    pub indirect_cost_rate_id: Option<Uuid>,
    pub indirect_cost_rate: String,
    pub cost_sharing_required: bool,
    pub cost_sharing_percent: String,
    pub principal_investigator_id: Option<Uuid>,
    pub principal_investigator_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub project_id: Option<Uuid>,
    pub cost_center: Option<String>,
    pub gl_revenue_account: Option<String>,
    pub gl_receivable_account: Option<String>,
    pub gl_deferred_account: Option<String>,
    pub billing_frequency: String,
    pub billing_basis: String,
    pub reporting_requirements: Option<String>,
    pub compliance_notes: Option<String>,
    pub closeout_date: Option<chrono::NaiveDate>,
    pub closeout_notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Grant Budget Line
/// Oracle Fusion: Grants Management > Award Budgets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantBudgetLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub award_id: Uuid,
    pub line_number: i32,
    pub budget_category: String,
    pub description: Option<String>,
    pub account_code: Option<String>,
    pub budget_amount: String,
    pub committed_amount: String,
    pub expended_amount: String,
    pub billed_amount: String,
    pub available_balance: String,
    pub period_start: Option<chrono::NaiveDate>,
    pub period_end: Option<chrono::NaiveDate>,
    pub fiscal_year: Option<i32>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Grant Expenditure
/// Oracle Fusion: Grants Management > Expenditures
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantExpenditure {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub award_id: Uuid,
    pub expenditure_number: String,
    pub expenditure_type: String,
    pub expenditure_date: chrono::NaiveDate,
    pub description: Option<String>,
    pub budget_line_id: Option<Uuid>,
    pub budget_category: Option<String>,
    pub amount: String,
    pub indirect_cost_amount: String,
    pub total_amount: String,
    pub cost_sharing_amount: String,
    pub employee_id: Option<Uuid>,
    pub employee_name: Option<String>,
    pub vendor_id: Option<Uuid>,
    pub vendor_name: Option<String>,
    pub source_entity_type: Option<String>,
    pub source_entity_id: Option<Uuid>,
    pub source_entity_number: Option<String>,
    pub gl_debit_account: Option<String>,
    pub gl_credit_account: Option<String>,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub billed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Grant Billing (invoice to sponsor)
/// Oracle Fusion: Grants Management > Billings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantBilling {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub award_id: Uuid,
    pub invoice_number: String,
    pub invoice_date: chrono::NaiveDate,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub direct_costs_billed: String,
    pub indirect_costs_billed: String,
    pub cost_sharing_billed: String,
    pub total_amount: String,
    pub amount_received: String,
    pub status: String,
    pub expenditure_ids: serde_json::Value,
    pub notes: Option<String>,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payment_reference: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Grant Compliance Report
/// Oracle Fusion: Grants Management > Compliance Reports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantComplianceReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub award_id: Uuid,
    pub report_type: String,
    pub report_title: Option<String>,
    pub reporting_period_start: chrono::NaiveDate,
    pub reporting_period_end: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub total_expenditures: String,
    pub total_billed: String,
    pub total_received: String,
    pub cash_draws: String,
    pub obligations: String,
    pub content: serde_json::Value,
    pub notes: Option<String>,
    pub prepared_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Corporate Card Management
// Oracle Fusion Cloud ERP: Financials > Expenses > Corporate Cards
// ============================================================================

/// Corporate Card Program – defines a card programme with an issuer bank.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCardProgram {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_code: String,
    pub name: String,
    pub description: Option<String>,
    pub issuer_bank: String,
    pub card_network: String,
    /// e.g. "corporate", "purchasing", "travel"
    pub card_type: String,
    pub currency_code: String,
    pub default_single_purchase_limit: String,
    pub default_monthly_limit: String,
    pub default_cash_limit: String,
    pub default_atm_limit: String,
    pub allow_cash_withdrawal: bool,
    pub allow_international: bool,
    pub auto_deactivate_on_termination: bool,
    pub expense_matching_method: String,
    pub billing_cycle_day: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Corporate Card – an individual card issued to an employee.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCard {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_id: Uuid,
    pub card_number_masked: String,
    pub cardholder_name: String,
    pub cardholder_id: Uuid,
    pub cardholder_email: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    /// "active", "suspended", "cancelled", "expired", "lost", "stolen"
    pub status: String,
    pub issue_date: chrono::NaiveDate,
    pub expiry_date: chrono::NaiveDate,
    pub single_purchase_limit: String,
    pub monthly_limit: String,
    pub cash_limit: String,
    pub atm_limit: String,
    pub current_balance: String,
    pub total_spend_current_cycle: String,
    pub last_statement_balance: String,
    pub last_statement_date: Option<chrono::NaiveDate>,
    pub gl_liability_account: Option<String>,
    pub gl_expense_account: Option<String>,
    pub cost_center: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Corporate Card Transaction – a charge or credit on a corporate card.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCardTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub card_id: Uuid,
    pub program_id: Uuid,
    pub transaction_reference: String,
    pub posting_date: chrono::NaiveDate,
    pub transaction_date: chrono::NaiveDate,
    pub merchant_name: String,
    pub merchant_category: Option<String>,
    pub merchant_category_code: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub original_amount: Option<String>,
    pub original_currency: Option<String>,
    pub exchange_rate: Option<String>,
    /// "charge", "credit", "payment", "cash_withdrawal", "fee", "interest"
    pub transaction_type: String,
    /// "unmatched", "matched", "disputed", "approved", "rejected"
    pub status: String,
    pub expense_report_id: Option<Uuid>,
    pub expense_line_id: Option<Uuid>,
    pub matched_at: Option<DateTime<Utc>>,
    pub matched_by: Option<Uuid>,
    pub match_confidence: Option<String>,
    pub dispute_reason: Option<String>,
    pub dispute_date: Option<chrono::NaiveDate>,
    pub dispute_resolution: Option<String>,
    pub gl_posted: bool,
    pub gl_journal_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Corporate Card Statement – a monthly billing statement from the card issuer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCardStatement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub program_id: Uuid,
    pub statement_number: String,
    pub statement_date: chrono::NaiveDate,
    pub billing_period_start: chrono::NaiveDate,
    pub billing_period_end: chrono::NaiveDate,
    pub opening_balance: String,
    pub closing_balance: String,
    pub total_charges: String,
    pub total_credits: String,
    pub total_payments: String,
    pub total_fees: String,
    pub total_interest: String,
    pub payment_due_date: Option<chrono::NaiveDate>,
    pub minimum_payment: String,
    pub total_transaction_count: i32,
    pub matched_transaction_count: i32,
    pub unmatched_transaction_count: i32,
    /// "imported", "processing", "matched", "reconciled", "paid"
    pub status: String,
    pub payment_reference: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub gl_payment_journal_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub imported_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Corporate Card Spending Limit Override – temporary or permanent limit changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCardLimitOverride {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub card_id: Uuid,
    pub override_type: String,
    pub original_value: String,
    pub new_value: String,
    pub reason: String,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    /// "pending", "approved", "rejected", "expired"
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Corporate Card Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CorporateCardDashboardSummary {
    pub total_active_cards: i32,
    pub total_programs: i32,
    pub total_cards_by_status: serde_json::Value,
    pub total_spend_current_month: String,
    pub total_spend_previous_month: String,
    pub spend_change_percent: String,
    pub total_unmatched_transactions: i32,
    pub total_unreconciled_statements: i32,
    pub total_disputed_transactions: i32,
    pub top_spenders: serde_json::Value,
    pub spend_by_category: serde_json::Value,
    pub limit_overrides_pending: i32,
}

/// Grant Management Dashboard Summary
/// Oracle Fusion: Grants Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantDashboardSummary {
    pub total_active_awards: i32,
    pub total_sponsors: i32,
    pub total_award_value: String,
    pub total_funded: String,
    pub total_expenditures: String,
    pub total_available_balance: String,
    pub total_pending_billings: i32,
    pub total_overdue_reports: i32,
    pub awards_expiring_30_days: i32,
    pub budget_utilization_percent: String,
    pub awards_by_status: serde_json::Value,
    pub expenditures_by_category: serde_json::Value,
    pub top_sponsors: serde_json::Value,
}

// ============================================================================
// Financial Consolidation (Oracle Fusion General Ledger > Consolidation)
// ============================================================================

/// Consolidation Ledger – defines a consolidation scope with translation method.
/// Oracle Fusion: General Ledger > Consolidation > Consolidation Ledgers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationLedger {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub base_currency_code: String,
    /// "current_rate", "temporal", "weighted_average"
    pub translation_method: String,
    /// "full", "proportional", "equity_method"
    pub equity_elimination_method: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consolidation Entity – a subsidiary / BU participating in consolidation.
/// Oracle Fusion: Consolidation > Consolidation Entities
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationEntity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ledger_id: Uuid,
    pub entity_id: Uuid,
    pub entity_name: String,
    pub entity_code: String,
    pub local_currency_code: String,
    pub ownership_percentage: String,
    /// "full", "proportional", "equity_method"
    pub consolidation_method: String,
    pub is_active: bool,
    pub include_in_consolidation: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consolidation Scenario – a periodic consolidation run.
/// Oracle Fusion: Consolidation > Consolidation Workbench
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationScenario {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ledger_id: Uuid,
    pub scenario_number: String,
    pub name: String,
    pub description: Option<String>,
    pub fiscal_year: i32,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    /// "draft", "in_progress", "pending_review", "approved", "posted", "reversed"
    pub status: String,
    pub translation_date: Option<chrono::NaiveDate>,
    pub translation_rate_type: Option<String>,
    pub total_entities: i32,
    pub total_eliminations: i32,
    pub total_adjustments: i32,
    pub total_debits: String,
    pub total_credits: String,
    pub is_balanced: bool,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consolidation Trial Balance Line.
/// Oracle Fusion: Consolidated Trial Balance report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationTrialBalanceLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scenario_id: Uuid,
    pub entity_id: Option<Uuid>,
    pub entity_code: Option<String>,
    pub account_code: String,
    pub account_name: Option<String>,
    pub account_type: Option<String>,
    pub financial_statement: Option<String>,
    pub local_debit: String,
    pub local_credit: String,
    pub local_balance: String,
    pub exchange_rate: Option<String>,
    pub translated_debit: String,
    pub translated_credit: String,
    pub translated_balance: String,
    pub elimination_debit: String,
    pub elimination_credit: String,
    pub elimination_balance: String,
    pub minority_interest_debit: String,
    pub minority_interest_credit: String,
    pub minority_interest_balance: String,
    pub consolidated_debit: String,
    pub consolidated_credit: String,
    pub consolidated_balance: String,
    pub is_elimination_entry: bool,
    /// "entity", "elimination", "adjustment", "minority", "consolidated"
    pub line_type: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Intercompany Elimination Rule.
/// Oracle Fusion: Consolidation > Elimination Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationEliminationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ledger_id: Uuid,
    pub rule_code: String,
    pub name: String,
    pub description: Option<String>,
    /// "intercompany_receivable_payable", "intercompany_revenue_expense",
    /// "investment_equity", "intercompany_inventory_profit", "other"
    pub elimination_type: String,
    pub from_entity_id: Option<Uuid>,
    pub to_entity_id: Option<Uuid>,
    pub from_account_pattern: Option<String>,
    pub to_account_pattern: Option<String>,
    pub offset_account_code: String,
    pub priority: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consolidation Adjustment – manual journal adjustment within a scenario.
/// Oracle Fusion: Consolidation > Adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationAdjustment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scenario_id: Uuid,
    pub adjustment_number: String,
    pub description: Option<String>,
    pub account_code: String,
    pub account_name: Option<String>,
    pub entity_id: Option<Uuid>,
    pub entity_code: Option<String>,
    pub debit: String,
    pub credit: String,
    /// "manual", "reclassification", "correction"
    pub adjustment_type: String,
    pub reference: Option<String>,
    /// "draft", "approved", "posted"
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Consolidation Currency Translation Rate.
/// Oracle Fusion: Consolidation > Translation Rates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationTranslationRate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scenario_id: Uuid,
    pub entity_id: Uuid,
    pub from_currency: String,
    pub to_currency: String,
    /// "period_end", "average", "historical", "spot"
    pub rate_type: String,
    pub exchange_rate: String,
    pub effective_date: chrono::NaiveDate,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Financial Consolidation Dashboard Summary.
/// Oracle Fusion: Consolidation > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsolidationDashboardSummary {
    pub total_ledgers: i32,
    pub total_active_scenarios: i32,
    pub total_entities: i32,
    pub total_elimination_rules: i32,
    pub last_consolidation_date: Option<String>,
    pub last_consolidation_status: Option<String>,
    pub scenarios_by_status: serde_json::Value,
    pub entities_by_method: serde_json::Value,
    pub consolidation_completion_percent: String,
}

// ============================================================================
// Supplier Qualification Management (Oracle Fusion Procurement > Supplier Qualification)
// ============================================================================

/// Qualification Area – defines a category of qualification criteria.
/// Oracle Fusion: Procurement > Supplier Qualification > Areas
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationArea {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub area_code: String,
    pub name: String,
    pub description: Option<String>,
    /// "questionnaire", "certificate", "financial", "site_visit", "reference", "other"
    pub area_type: String,
    /// "manual", "weighted", "pass_fail"
    pub scoring_model: String,
    pub passing_score: String,
    pub is_mandatory: bool,
    pub renewal_period_days: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Qualification Question – individual question within an area.
/// Oracle Fusion: Procurement > Supplier Qualification > Questions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualificationQuestion {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub area_id: Uuid,
    pub question_number: i32,
    pub question_text: String,
    pub description: Option<String>,
    /// "text", "yes_no", "numeric", "date", "multi_choice", "file_upload"
    pub response_type: String,
    pub choices: Option<serde_json::Value>,
    pub is_required: bool,
    pub weight: String,
    pub max_score: String,
    pub help_text: Option<String>,
    pub display_order: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Qualification Initiative – a qualification campaign/run.
/// Oracle Fusion: Procurement > Supplier Qualification > Initiatives
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierQualificationInitiative {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub initiative_number: String,
    pub name: String,
    pub description: Option<String>,
    pub area_id: Uuid,
    /// "new_supplier", "requalification", "compliance", "ad_hoc"
    pub qualification_purpose: String,
    /// "draft", "active", "pending_evaluations", "completed", "cancelled"
    pub status: String,
    pub deadline: Option<chrono::NaiveDate>,
    pub total_invited: i32,
    pub total_responded: i32,
    pub total_qualified: i32,
    pub total_disqualified: i32,
    pub total_pending: i32,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Qualification Invitation – per-supplier qualification within an initiative.
/// Oracle Fusion: Procurement > Supplier Qualification > Invitations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierQualificationInvitation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub initiative_id: Uuid,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub supplier_contact_name: Option<String>,
    pub supplier_contact_email: Option<String>,
    /// "initiated", "pending_response", "under_evaluation", "qualified", "disqualified", "expired", "withdrawn"
    pub status: String,
    pub invitation_date: Option<DateTime<Utc>>,
    pub response_date: Option<DateTime<Utc>>,
    pub evaluation_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub overall_score: String,
    pub max_possible_score: String,
    pub score_percentage: String,
    pub qualified_by: Option<Uuid>,
    pub disqualified_reason: Option<String>,
    pub evaluation_notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Qualification Response – individual answer to a question.
/// Oracle Fusion: Procurement > Supplier Qualification > Responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierQualificationResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invitation_id: Uuid,
    pub question_id: Uuid,
    pub response_text: Option<String>,
    pub response_value: Option<serde_json::Value>,
    pub file_reference: Option<String>,
    pub score: String,
    pub max_score: String,
    pub evaluator_notes: Option<String>,
    pub evaluated_by: Option<Uuid>,
    pub evaluated_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Certification – track ongoing supplier certifications.
/// Oracle Fusion: Procurement > Supplier Qualification > Certifications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierCertification {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub supplier_id: Uuid,
    pub supplier_name: String,
    pub certification_type: String,
    pub certification_name: String,
    pub certifying_body: Option<String>,
    pub certificate_number: Option<String>,
    /// "active", "expired", "revoked", "pending_renewal"
    pub status: String,
    pub issued_date: Option<chrono::NaiveDate>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub renewal_date: Option<chrono::NaiveDate>,
    pub qualification_invitation_id: Option<Uuid>,
    pub document_reference: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Qualification Dashboard Summary.
/// Oracle Fusion: Procurement > Supplier Qualification > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierQualificationDashboardSummary {
    pub total_active_areas: i32,
    pub total_active_initiatives: i32,
    pub total_suppliers_invited: i32,
    pub total_suppliers_qualified: i32,
    pub total_suppliers_pending: i32,
    pub total_suppliers_disqualified: i32,
    pub total_certifications_active: i32,
    pub total_certifications_expiring_30_days: i32,
    pub qualification_rate_percent: String,
    pub initiatives_by_status: serde_json::Value,
    pub certifications_by_type: serde_json::Value,
}

// ============================================================================
// Recurring Journals (Oracle Fusion GL > Recurring Journals)
// ============================================================================

/// Recurring journal schedule definition.
/// Oracle Fusion: General Ledger > Journals > Recurring Journals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringJournalSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_number: String,
    pub name: String,
    pub description: Option<String>,
    pub recurrence_type: String,       // daily, weekly, monthly, quarterly, semi_annual, annual
    pub journal_type: String,          // standard, skeleton, incremental
    pub currency_code: String,
    pub status: String,                // draft, active, inactive
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub last_generation_date: Option<chrono::NaiveDate>,
    pub next_generation_date: Option<chrono::NaiveDate>,
    pub total_generations: i32,
    pub incremental_percent: Option<String>,
    pub auto_post: bool,
    pub reversal_method: Option<String>,
    pub ledger_id: Option<Uuid>,
    pub journal_category: Option<String>,
    pub reference_template: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A template line within a recurring journal schedule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringJournalScheduleLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_id: Uuid,
    pub line_number: i32,
    pub line_type: String,             // debit, credit
    pub account_code: String,
    pub account_name: Option<String>,
    pub description: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub tax_code: Option<String>,
    pub cost_center: Option<String>,
    pub department_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single generation run of a recurring journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringJournalGeneration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_id: Uuid,
    pub generation_number: i32,
    pub journal_entry_id: Option<Uuid>,
    pub journal_entry_number: Option<String>,
    pub generation_date: chrono::NaiveDate,
    pub period_name: Option<String>,
    pub total_debit: String,
    pub total_credit: String,
    pub line_count: i32,
    pub status: String,               // generated, posted, reversed, cancelled
    pub reversal_entry_id: Option<Uuid>,
    pub reversed_at: Option<DateTime<Utc>>,
    pub posted_at: Option<DateTime<Utc>>,
    pub generated_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single generated journal line.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringJournalGenerationLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub generation_id: Uuid,
    pub schedule_line_id: Option<Uuid>,
    pub line_number: i32,
    pub line_type: String,
    pub account_code: String,
    pub account_name: Option<String>,
    pub description: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub tax_code: Option<String>,
    pub cost_center: Option<String>,
    pub department_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Recurring Journals Dashboard Summary.
/// Oracle Fusion: General Ledger > Recurring Journals > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringJournalDashboardSummary {
    pub total_active_schedules: i32,
    pub total_draft_schedules: i32,
    pub total_generations: i32,
    pub total_generations_this_month: i32,
    pub total_generated_amount: String,
    pub schedules_due_today: i32,
    pub schedules_overdue: i32,
    pub schedules_by_recurrence: serde_json::Value,
    pub schedules_by_status: serde_json::Value,
    pub recent_generations: Vec<RecurringJournalGeneration>,
}

// ============================================================================
// Manual Journal Entries (Oracle Fusion GL > Journals > New Journal)
// ============================================================================

/// A journal batch that groups multiple journal entries.
/// Oracle Fusion: General Ledger > Journals > Journal Batch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub ledger_id: Option<Uuid>,
    pub currency_code: String,
    pub accounting_date: Option<chrono::NaiveDate>,
    pub period_name: Option<String>,
    pub total_debit: String,
    pub total_credit: String,
    pub entry_count: i32,
    pub source: String,
    pub is_automatic_post: bool,
    pub submitted_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,
    pub posted_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A journal entry within a batch, containing debit/credit lines.
/// Oracle Fusion: General Ledger > Journals > Journal Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub entry_number: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub ledger_id: Option<Uuid>,
    pub currency_code: String,
    pub accounting_date: Option<chrono::NaiveDate>,
    pub period_name: Option<String>,
    pub journal_category: String,
    pub journal_source: String,
    pub total_debit: String,
    pub total_credit: String,
    pub line_count: i32,
    pub is_balanced: bool,
    pub is_reversal: bool,
    pub reversal_of_entry_id: Option<Uuid>,
    pub reversed_by_entry_id: Option<Uuid>,
    pub reference_number: Option<String>,
    pub external_reference: Option<String>,
    pub statistical_entry: bool,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub posted_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A journal entry line (debit or credit).
/// Oracle Fusion: General Ledger > Journals > Journal Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entry_id: Uuid,
    pub line_number: i32,
    pub line_type: String,
    pub account_code: String,
    pub account_name: Option<String>,
    pub description: Option<String>,
    pub amount: String,
    pub entered_amount: Option<String>,
    pub entered_currency_code: Option<String>,
    pub exchange_rate: Option<String>,
    pub tax_code: Option<String>,
    pub cost_center: Option<String>,
    pub department_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub intercompany_entity_id: Option<Uuid>,
    pub statistical_amount: Option<String>,
    pub reference1: Option<String>,
    pub reference2: Option<String>,
    pub reference3: Option<String>,
    pub reference4: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Manual Journal Entries Dashboard Summary.
/// Oracle Fusion: General Ledger > Journals > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManualJournalDashboardSummary {
    pub total_batches: i32,
    pub total_draft_batches: i32,
    pub total_posted_batches: i32,
    pub total_entries: i32,
    pub total_posted_entries: i32,
    pub total_debits: String,
    pub total_credits: String,
    pub batches_pending_approval: i32,
    pub entries_by_category: serde_json::Value,
    pub batches_by_status: serde_json::Value,
    pub recent_batches: Vec<JournalBatch>,
}

// ============================================================================
// Document Sequencing
// Oracle Fusion: General Ledger > Setup > Document Sequencing
// ============================================================================

/// A document sequence definition.
/// Controls automatic numbering of business documents (invoices, POs, journals, etc.)
/// Supports gapless (regulatory compliance) and gap-permitted (operational) modes.
/// Oracle Fusion: General Ledger > Setup > Sequences > Document Sequences
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSequence {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub sequence_type: String, // gapless, gap_permitted, manual
    pub document_type: String, // invoice, purchase_order, journal_entry, payment, receipt, etc.
    pub initial_value: i64,
    pub current_value: i64,
    pub increment_by: i32,
    pub max_value: Option<i64>,
    pub cycle_flag: bool,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub pad_length: i32,
    pub pad_character: String,
    pub reset_frequency: Option<String>, // daily, monthly, quarterly, annually, never
    pub last_reset_date: Option<chrono::NaiveDate>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String, // active, inactive
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A sequence assignment that maps a document sequence to a specific
/// document category + business unit + ledger combination.
/// Oracle Fusion: General Ledger > Setup > Sequences > Sequence Assignments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSequenceAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub sequence_id: Uuid,
    pub sequence_code: String,
    pub document_category: String, // e.g. "accounts_payable_invoice", "gl_journal"
    pub business_unit_id: Option<Uuid>,
    pub ledger_id: Option<Uuid>,
    pub method: String, // automatic, manual
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub priority: i32,
    pub status: String, // active, inactive
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An audit record for a generated document number.
/// Tracks every number assignment for compliance and traceability.
/// Oracle Fusion: General Ledger > Sequences > Sequence Audit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSequenceAudit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub sequence_id: Uuid,
    pub sequence_code: String,
    pub generated_number: String,
    pub numeric_value: i64,
    pub document_category: String,
    pub document_id: Option<Uuid>,
    pub document_number: Option<String>,
    pub business_unit_id: Option<Uuid>,
    pub generated_at: DateTime<Utc>,
    pub generated_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Summary statistics for document sequences.
/// Oracle Fusion: General Ledger > Sequences > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSequenceDashboardSummary {
    pub total_sequences: i32,
    pub active_sequences: i32,
    pub gapless_sequences: i32,
    pub gap_permitted_sequences: i32,
    pub total_numbers_generated: i64,
    pub total_assignments: i32,
    pub recent_audits: Vec<DocumentSequenceAudit>,
    pub sequences_by_type: serde_json::Value,
    pub sequences_by_document_type: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Descriptive Flexfields (Oracle Fusion DFF)
// ═══════════════════════════════════════════════════════════════════════════════
//
// Descriptive Flexfields allow administrators to add custom configurable
// fields to any entity at runtime. They support:
// - Global segments (always visible on the entity)
// - Context-sensitive segments (visible based on a context value)
// - Value sets with validation rules (table-validated, independent, dependent, etc.)
// - Required / optional segments with defaults
//
// Oracle Fusion equivalent: Application Extensions > Flexfields > Descriptive
// ═══════════════════════════════════════════════════════════════════════════════

/// A value set defines the list of valid values for a flexfield segment.
/// Value sets enforce validation at data entry time.
///
/// Oracle Fusion: Setup and Maintenance > Manage Value Sets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldValueSet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Type of validation: "none", "independent", "dependent", "table", "format_only"
    pub validation_type: String,
    /// Data type for values: "string", "number", "date", "datetime"
    pub data_type: String,
    /// Maximum length for string values
    pub max_length: i32,
    /// Minimum length for string values
    pub min_length: i32,
    /// For format_only: regex or format pattern
    pub format_mask: Option<String>,
    /// For table validation: table name, value column, meaning column, WHERE clause
    pub table_validation: Option<serde_json::Value>,
    /// For independent: list of valid values stored as JSON
    pub independent_values: Option<serde_json::Value>,
    /// For dependent: reference to parent value set code
    pub parent_value_set_code: Option<String>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A value set value entry (for independent and dependent value sets).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldValueSetEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub value_set_id: Uuid,
    pub value: String,
    pub meaning: Option<String>,
    pub description: Option<String>,
    /// For dependent value sets: the parent value this depends on
    pub parent_value: Option<String>,
    pub is_enabled: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub sort_order: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A descriptive flexfield definition attached to an entity.
///
/// Oracle Fusion: Application Extensions > Flexfields > Descriptive Flexfields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptiveFlexfield {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// The entity/table this flexfield is attached to (e.g., "purchase_orders")
    pub entity_name: String,
    /// The column on the entity table where the context value is stored (default: "dff_context")
    pub context_column: String,
    /// Default context code used when no context is specified
    pub default_context_code: Option<String>,
    /// Whether the flexfield is active
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A context within a flexfield. Contexts group segments and allow
/// context-sensitive custom fields (different fields appear based on context).
///
/// Oracle Fusion: Each DFF can have multiple contexts (e.g., "US", "EU" for
/// region-specific fields, or "IT", "Hardware" for category-specific fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldContext {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub flexfield_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Whether this is a global context (segments apply to all records)
    pub is_global: bool,
    pub is_enabled: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A segment (custom field) within a flexfield context.
/// Segments are ordered and can be required or optional.
///
/// Oracle Fusion: Each context has 1-N segments with display ordering
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldSegment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub flexfield_id: Uuid,
    pub context_id: Uuid,
    pub segment_code: String,
    pub name: String,
    pub description: Option<String>,
    /// Display order within the context
    pub display_order: i32,
    /// The column used to store this segment's value (e.g., "attribute1", "attribute2")
    pub column_name: String,
    /// Data type: "string", "number", "date", "datetime"
    pub data_type: String,
    /// Whether this segment is required
    pub is_required: bool,
    /// Whether this segment is read-only
    pub is_read_only: bool,
    /// Whether this segment is visible on forms
    pub is_visible: bool,
    /// Default value for the segment
    pub default_value: Option<String>,
    /// Reference to the value set for validation (by ID)
    pub value_set_id: Option<Uuid>,
    /// Reference to the value set for validation (by code, denormalized)
    pub value_set_code: Option<String>,
    /// Help text shown to users
    pub help_text: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Flexfield data values stored for a specific entity record.
/// The context value plus all segment values for that context.
///
/// Oracle Fusion: DFF values are stored in attribute columns on the entity row
/// or in a dedicated flexfield values table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldData {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub flexfield_id: Uuid,
    pub entity_name: String,
    pub entity_id: Uuid,
    /// The context code selected for this record
    pub context_code: String,
    /// Segment values as a JSON object: {"segment_code": "value", ...}
    pub segment_values: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dashboard summary for descriptive flexfields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexfieldDashboardSummary {
    pub total_flexfields: i32,
    pub active_flexfields: i32,
    pub total_contexts: i32,
    pub total_segments: i32,
    pub total_value_sets: i32,
    pub flexfields_by_entity: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cross-Validation Rules (Oracle Fusion GL > Chart of Accounts > Cross-Validation)
// ═══════════════════════════════════════════════════════════════════════════════
//
// Cross-Validation Rules prevent users from creating invalid combinations
// of account segment values. For example, you can prevent creating an
// account like "1000.Cash.4000.Marketing" if that combination doesn't
// make business sense.
//
// Each rule defines a pattern of segment values that must (or must not)
// co-occur. Patterns use exact values or wildcards ("%" = any value,
// "T" = typed shorthand, etc.).
//
// Oracle Fusion equivalent: General Ledger > Setup > Chart of Accounts >
//   Cross-Validation Rules
// ═══════════════════════════════════════════════════════════════════════════════

/// A Cross-Validation Rule (CVR) definition.
/// Each rule has a name, an enabled flag, effective dates, an error message,
/// and a type ("allow" or "deny").
///
/// Oracle Fusion: General Ledger > Setup > Chart of Accounts > Cross-Validation Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossValidationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code for the rule, e.g. "CVR_CASH_MARKETING"
    pub code: String,
    /// Human-readable name
    pub name: String,
    /// Description of the rule
    pub description: Option<String>,
    /// Rule type: "deny" blocks matching combinations, "allow" permits them
    pub rule_type: String,
    /// Error message shown when the rule is violated
    pub error_message: String,
    /// Whether this rule is enabled
    pub is_enabled: bool,
    /// Priority order (lower = evaluated first)
    pub priority: i32,
    /// Segment names in the chart of accounts (e.g. ["company", "department", "account"])
    pub segment_names: Vec<String>,
    /// Effective from date
    pub effective_from: Option<chrono::NaiveDate>,
    /// Effective to date
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A validation line within a Cross-Validation Rule.
/// Each line specifies a pattern for the account segment combination.
/// Patterns use exact values or "%" as a wildcard meaning "any value".
///
/// Example ("deny" rule with 2 lines):
///   Line 1 (from): {"patterns": ["1000", "%", "%"]}  -- company=1000
///   Line 2 (to):   {"patterns": ["%", "%", "5000"]}  -- account=5000
///   Meaning: Deny any combination where company=1000 AND account=5000
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossValidationRuleLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rule_id: Uuid,
    /// Line type: "from" (left side) or "to" (right side)
    pub line_type: String,
    /// Pattern values for each segment. "%" = any value, exact string = exact match
    pub patterns: Vec<String>,
    /// Display order
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Result of a cross-validation check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossValidationResult {
    /// Whether the combination is valid (passes all rules)
    pub is_valid: bool,
    /// List of violated rule codes
    pub violated_rules: Vec<String>,
    /// List of error messages from violated rules
    pub error_messages: Vec<String>,
}

/// Dashboard summary for Cross-Validation Rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossValidationDashboardSummary {
    pub total_rules: i32,
    pub enabled_rules: i32,
    pub deny_rules: i32,
    pub allow_rules: i32,
    pub total_lines: i32,
    pub rules_by_type: serde_json::Value,
}

// ============================================================================
// Scheduled Processes (Oracle Fusion Enterprise Scheduler Service)
// ============================================================================
// Oracle Fusion: Navigator > Tools > Scheduled Processes
// Allows users to submit, schedule, and monitor batch processes, reports,
// data imports/exports, and custom jobs.

/// Process template definition
/// Oracle Fusion: Scheduled Processes > Manage Process Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledProcessTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub process_type: String,
    pub executor_type: String,
    pub executor_config: serde_json::Value,
    pub parameters: serde_json::Value,
    pub default_parameters: serde_json::Value,
    pub timeout_minutes: i32,
    pub max_retries: i32,
    pub retry_delay_minutes: i32,
    pub requires_approval: bool,
    pub approval_chain_id: Option<Uuid>,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Submitted process instance
/// Oracle Fusion: Scheduled Processes > Submit New Process / Monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledProcess {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Option<Uuid>,
    pub template_code: Option<String>,
    pub process_name: String,
    pub process_type: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub submitted_by: Uuid,
    pub submitted_at: DateTime<Utc>,
    pub scheduled_start_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancelled_by: Option<Uuid>,
    pub cancel_reason: Option<String>,
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub timeout_minutes: i32,
    pub progress_percent: i32,
    pub parameters: serde_json::Value,
    pub result_summary: Option<String>,
    pub output_file_url: Option<String>,
    pub output_format: String,
    pub log_output: Option<String>,
    pub error_message: Option<String>,
    pub parent_process_id: Option<Uuid>,
    pub recurrence_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Recurrence schedule for recurring process submissions
/// Oracle Fusion: Scheduled Processes > Schedule > Recurrence
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledProcessRecurrence {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub template_id: Uuid,
    pub template_code: Option<String>,
    pub parameters: serde_json::Value,
    pub recurrence_type: String,
    pub recurrence_config: serde_json::Value,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub max_runs: Option<i32>,
    pub is_active: bool,
    pub submitted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Process execution log entry
/// Oracle Fusion: Scheduled Processes > Process Details > Log
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledProcessLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub process_id: Uuid,
    pub log_level: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub step_name: Option<String>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Scheduled processes dashboard summary
/// Oracle Fusion: Scheduled Processes > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledProcessDashboardSummary {
    pub total_processes: i32,
    pub pending_processes: i32,
    pub running_processes: i32,
    pub completed_processes: i32,
    pub failed_processes: i32,
    pub cancelled_processes: i32,
    pub scheduled_processes: i32,
    pub active_recurrences: i32,
    pub recent_processes: Vec<ScheduledProcess>,
    pub processes_by_type: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════
// Segregation of Duties (SoD)
// Oracle Fusion: Advanced Access Control > Segregation of Duties
// ═══════════════════════════════════════════════════════════════════════════

/// An SoD rule defines a pair (or set) of incompatible duties/roles.
/// For example: "Create Vendor" and "Approve Vendor Payments" must not be
/// held by the same person.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// The set of duties that conflict with each other.
    /// A violation occurs when a single user holds duties from
    /// *both* `first_duties` AND `second_duties`.
    pub first_duties: Vec<String>,
    pub second_duties: Vec<String>,
    /// "preventive" = block violating role assignments,
    /// "detective" = report but allow
    pub enforcement_mode: String,
    /// Risk level: "high", "medium", "low"
    pub risk_level: String,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An SoD violation detected for a specific user against a rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodViolation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rule_id: Uuid,
    pub rule_code: String,
    pub user_id: Uuid,
    /// The duties from the first set the user holds
    pub first_matched_duties: Vec<String>,
    /// The duties from the second set the user holds
    pub second_matched_duties: Vec<String>,
    pub violation_status: String, // "open", "mitigated", "exception", "resolved"
    pub detected_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A mitigating control applied to an SoD violation.
/// In Oracle Fusion, this is a documented compensating control that
/// reduces the risk of the conflict to an acceptable level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodMitigatingControl {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub violation_id: Uuid,
    pub control_name: String,
    pub control_description: String,
    /// Who is responsible for executing this control
    pub control_owner_id: Option<Uuid>,
    /// Frequency: "daily", "weekly", "monthly", "quarterly"
    pub review_frequency: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub status: String, // "active", "expired", "revoked"
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Role assignment entry tracked for SoD analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodRoleAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub role_name: String,
    /// The duty/privilege this role grants
    pub duty_code: String,
    pub assigned_by: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Result of checking a proposed role assignment for conflicts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodConflictCheckResult {
    pub has_conflicts: bool,
    pub conflicts: Vec<SodConflictDetail>,
    pub would_be_blocked: bool,
}

/// Details of a single conflict found during a check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodConflictDetail {
    pub rule_id: Uuid,
    pub rule_code: String,
    pub rule_name: String,
    pub risk_level: String,
    pub enforcement_mode: String,
    pub conflicting_duty: String,
    pub existing_duties_causing_conflict: Vec<String>,
}

/// Dashboard summary for SoD compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SodDashboardSummary {
    pub total_rules: i32,
    pub active_rules: i32,
    pub total_violations: i32,
    pub open_violations: i32,
    pub mitigated_violations: i32,
    pub exception_violations: i32,
    pub violations_by_risk_level: serde_json::Value,
    pub recent_violations: Vec<SodViolation>,
    pub rules_summary: serde_json::Value,
}

// ============================================================================
// General Ledger Allocations (Oracle Fusion GL Allocations)
// ============================================================================

/// GL Allocation pool definition
/// Oracle Fusion: General Ledger > Allocations > Allocation Pools
/// Represents a pool of costs/amounts to be distributed to targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationPool {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code for this pool (e.g., "RENT_POOL", "IT_OVERHEAD")
    pub code: String,
    /// Human-readable name
    pub name: String,
    pub description: Option<String>,
    /// Pool type: 'cost_center', 'account_range', 'manual'
    pub pool_type: String,
    /// Source account code or range for the pool
    pub source_account_code: Option<String>,
    pub source_account_range_from: Option<String>,
    pub source_account_range_to: Option<String>,
    /// Source cost center or department
    pub source_department_id: Option<Uuid>,
    /// Source project
    pub source_project_id: Option<Uuid>,
    /// Currency code
    pub currency_code: String,
    /// Whether this pool is active
    pub is_active: bool,
    /// Start/end dates for effective period
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update GL allocation pool request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationPoolRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_pool_type")]
    pub pool_type: String,
    pub source_account_code: Option<String>,
    pub source_account_range_from: Option<String>,
    pub source_account_range_to: Option<String>,
    pub source_department_id: Option<Uuid>,
    pub source_project_id: Option<Uuid>,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_pool_type() -> String { "cost_center".to_string() }

/// GL Allocation basis definition
/// Oracle Fusion: General Ledger > Allocations > Allocation Bases
/// Defines how amounts are distributed (e.g., headcount, revenue, square footage).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationBasis {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code (e.g., "HEADCOUNT", "REVENUE", "SQFT")
    pub code: String,
    /// Human-readable name
    pub name: String,
    pub description: Option<String>,
    /// Basis type: 'statistical', 'financial', 'percentage'
    pub basis_type: String,
    /// Unit of measure (e.g., 'people', 'USD', 'sq_ft')
    pub unit_of_measure: Option<String>,
    /// Whether the basis amounts are entered manually or sourced from GL
    pub is_manual: bool,
    /// GL account code to source basis amounts from (if not manual)
    pub source_account_code: Option<String>,
    /// Whether this basis is active
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update GL allocation basis request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationBasisRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_basis_type")]
    pub basis_type: String,
    pub unit_of_measure: Option<String>,
    #[serde(default)]
    pub is_manual: bool,
    pub source_account_code: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

fn default_basis_type() -> String { "statistical".to_string() }

/// GL Allocation basis detail (individual target's share of the basis)
/// Oracle Fusion: General Ledger > Allocations > Basis Details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationBasisDetail {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// The basis this detail belongs to
    pub basis_id: Uuid,
    /// Target dimension: department, cost center, project, or account
    pub target_department_id: Option<Uuid>,
    pub target_department_name: Option<String>,
    pub target_cost_center: Option<String>,
    pub target_project_id: Option<Uuid>,
    pub target_project_name: Option<String>,
    pub target_account_code: Option<String>,
    /// The basis amount (headcount count, revenue amount, square footage, etc.)
    pub basis_amount: String,
    /// The percentage this detail represents of the total basis
    pub percentage: String,
    /// Effective period
    pub period_name: Option<String>,
    pub period_start_date: Option<chrono::NaiveDate>,
    pub period_end_date: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update GL allocation basis detail request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationBasisDetailRequest {
    pub target_department_id: Option<Uuid>,
    pub target_department_name: Option<String>,
    pub target_cost_center: Option<String>,
    pub target_project_id: Option<Uuid>,
    pub target_project_name: Option<String>,
    pub target_account_code: Option<String>,
    pub basis_amount: String,
    pub period_name: Option<String>,
    pub period_start_date: Option<chrono::NaiveDate>,
    pub period_end_date: Option<chrono::NaiveDate>,
}

/// GL Allocation rule definition
/// Oracle Fusion: General Ledger > Allocations > Allocation Rules
/// Maps a pool to targets using a basis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    /// Unique code (e.g., "RENT_ALLOC", "IT_OVERHEAD_ALLOC")
    pub code: String,
    /// Human-readable name
    pub name: String,
    pub description: Option<String>,
    /// The pool to allocate from
    pub pool_id: Uuid,
    pub pool_code: String,
    /// The basis to use for distribution
    pub basis_id: Uuid,
    pub basis_code: String,
    /// Allocation method: 'proportional', 'fixed_percentage', 'step_down'
    pub allocation_method: String,
    /// Offset method: 'none', 'same_account', 'specified_account'
    pub offset_method: String,
    /// Offset account for the credit side of the allocation
    pub offset_account_code: Option<String>,
    /// Journal batch name prefix for generated entries
    pub journal_batch_prefix: Option<String>,
    /// Whether allocations round differences to the largest target
    pub round_to_largest: bool,
    /// Minimum allocation threshold (amounts below this are not allocated)
    pub minimum_threshold: Option<String>,
    /// Whether this rule is active
    pub is_active: bool,
    /// Effective dates
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    /// Target lines for this allocation rule
    pub target_lines: Vec<GlAllocationTargetLine>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// GL Allocation target line
/// Oracle Fusion: General Ledger > Allocations > Target Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationTargetLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub rule_id: Uuid,
    pub line_number: i32,
    pub target_department_id: Option<Uuid>,
    pub target_department_name: Option<String>,
    pub target_cost_center: Option<String>,
    pub target_project_id: Option<Uuid>,
    pub target_project_name: Option<String>,
    pub target_account_code: String,
    pub target_account_name: Option<String>,
    pub fixed_percentage: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create/update GL allocation rule request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationRuleRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub pool_code: String,
    pub basis_code: String,
    #[serde(default = "default_gl_allocation_method")]
    pub allocation_method: String,
    #[serde(default = "default_gl_offset_method")]
    pub offset_method: String,
    pub offset_account_code: Option<String>,
    pub journal_batch_prefix: Option<String>,
    #[serde(default)]
    pub round_to_largest: bool,
    pub minimum_threshold: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub target_lines: Option<Vec<GlAllocationTargetLineRequest>>,
}

fn default_gl_allocation_method() -> String { "proportional".to_string() }
fn default_gl_offset_method() -> String { "same_account".to_string() }

/// Create/update GL allocation target line request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationTargetLineRequest {
    pub target_department_id: Option<Uuid>,
    pub target_department_name: Option<String>,
    pub target_cost_center: Option<String>,
    pub target_project_id: Option<Uuid>,
    pub target_project_name: Option<String>,
    pub target_account_code: String,
    pub target_account_name: Option<String>,
    pub fixed_percentage: Option<String>,
    pub is_active: Option<bool>,
}

/// GL Allocation run (generated journal entry batch)
/// Oracle Fusion: General Ledger > Allocations > Allocation Runs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_number: String,
    pub rule_id: Uuid,
    pub rule_code: String,
    pub rule_name: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub pool_amount: String,
    pub allocation_method: String,
    pub total_allocated: String,
    pub total_offset: String,
    pub rounding_difference: String,
    pub target_count: i32,
    pub journal_batch_id: Option<Uuid>,
    pub journal_batch_name: Option<String>,
    pub status: String,
    pub run_date: chrono::NaiveDate,
    pub posted_at: Option<DateTime<Utc>>,
    pub reversed_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,
    pub results: Vec<GlAllocationRunLine>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual line in a GL allocation run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationRunLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub line_number: i32,
    pub target_department_id: Option<Uuid>,
    pub target_department_name: Option<String>,
    pub target_cost_center: Option<String>,
    pub target_project_id: Option<Uuid>,
    pub target_project_name: Option<String>,
    pub target_account_code: String,
    pub target_account_name: Option<String>,
    pub source_account_code: Option<String>,
    pub basis_amount: String,
    pub basis_percentage: String,
    pub allocated_amount: String,
    pub offset_amount: String,
    pub line_type: String,
    pub journal_line_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// GL Allocation run request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationRunRequest {
    pub rule_code: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub run_date: Option<chrono::NaiveDate>,
    pub pool_amount_override: Option<String>,
}

/// GL Allocation dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlAllocationDashboardSummary {
    pub total_pools: i32,
    pub active_pools: i32,
    pub total_bases: i32,
    pub active_bases: i32,
    pub total_rules: i32,
    pub active_rules: i32,
    pub total_runs: i32,
    pub posted_runs: i32,
    pub draft_runs: i32,
    pub total_allocated_amount: String,
    pub pools_by_type: serde_json::Value,
    pub rules_by_method: serde_json::Value,
}

// ============================================================================
// Currency Revaluation Types
// Oracle Fusion Cloud ERP: General Ledger > Currency Revaluation
// ============================================================================

/// Currency Revaluation Definition
/// Defines which accounts to revalue, what rate to use, and where to post gains/losses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub revaluation_type: String,
    pub currency_code: String,
    pub rate_type: String,
    pub gain_account_code: String,
    pub loss_account_code: String,
    pub unrealized_gain_account_code: Option<String>,
    pub unrealized_loss_account_code: Option<String>,
    pub account_range_from: Option<String>,
    pub account_range_to: Option<String>,
    pub include_subledger: bool,
    pub auto_reverse: bool,
    pub reversal_period_offset: i32,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub accounts: Vec<CurrencyRevaluationAccount>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Account included in a revaluation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationAccount {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub definition_id: Uuid,
    pub account_code: String,
    pub account_name: Option<String>,
    pub account_type: String,
    pub is_included: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Currency Revaluation Run
/// A batch execution of a revaluation definition for a specific period
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_number: String,
    pub definition_id: Uuid,
    pub definition_code: String,
    pub definition_name: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub revaluation_date: chrono::NaiveDate,
    pub currency_code: String,
    pub rate_type: String,
    pub total_revalued_amount: String,
    pub total_gain_amount: String,
    pub total_loss_amount: String,
    pub total_entries: i32,
    pub status: String,
    pub reversal_run_id: Option<Uuid>,
    pub original_run_id: Option<Uuid>,
    pub reversed_at: Option<DateTime<Utc>>,
    pub posted_at: Option<DateTime<Utc>>,
    pub posted_by: Option<Uuid>,
    pub lines: Vec<CurrencyRevaluationLine>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual line in a revaluation run (one per account)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub run_id: Uuid,
    pub line_number: i32,
    pub account_code: String,
    pub account_name: Option<String>,
    pub account_type: String,
    pub original_amount: String,
    pub original_currency: String,
    pub original_exchange_rate: String,
    pub original_base_amount: String,
    pub revalued_exchange_rate: String,
    pub revalued_base_amount: String,
    pub gain_loss_amount: String,
    pub gain_loss_type: String,
    pub gain_loss_account_code: String,
    pub reversal_line_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create revaluation definition request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationDefinitionRequest {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_revaluation_type")]
    pub revaluation_type: String,
    pub currency_code: String,
    #[serde(default = "default_rate_type_period_end")]
    pub rate_type: String,
    pub gain_account_code: String,
    pub loss_account_code: String,
    pub unrealized_gain_account_code: Option<String>,
    pub unrealized_loss_account_code: Option<String>,
    pub account_range_from: Option<String>,
    pub account_range_to: Option<String>,
    #[serde(default)]
    pub include_subledger: bool,
    #[serde(default = "default_true")]
    pub auto_reverse: bool,
    #[serde(default = "default_one")]
    pub reversal_period_offset: i32,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub accounts: Option<Vec<CurrencyRevaluationAccountRequest>>,
}

fn default_revaluation_type() -> String { "period_end".to_string() }
fn default_rate_type_period_end() -> String { "period_end".to_string() }
fn default_account_type_asset() -> String { "asset".to_string() }

/// Create revaluation account request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationAccountRequest {
    pub account_code: String,
    pub account_name: Option<String>,
    #[serde(default = "default_account_type_asset")]
    pub account_type: String,
    #[serde(default = "default_true")]
    pub is_included: bool,
}

/// Execute revaluation run request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationRunRequest {
    pub definition_code: String,
    pub period_name: String,
    pub period_start_date: chrono::NaiveDate,
    pub period_end_date: chrono::NaiveDate,
    pub revaluation_date: Option<chrono::NaiveDate>,
    pub rate_type_override: Option<String>,
    pub balances: Vec<CurrencyRevaluationBalanceRequest>,
}

/// Balance input for revaluation run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationBalanceRequest {
    pub account_code: String,
    pub account_name: Option<String>,
    pub account_type: String,
    pub original_amount: String,
    pub original_currency: String,
    pub original_exchange_rate: String,
    pub original_base_amount: String,
}

/// Currency Revaluation dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyRevaluationDashboardSummary {
    pub total_definitions: i32,
    pub active_definitions: i32,
    pub total_runs: i32,
    pub posted_runs: i32,
    pub draft_runs: i32,
    pub reversed_runs: i32,
    pub total_gain_amount: String,
    pub total_loss_amount: String,
    pub definitions_by_type: serde_json::Value,
}

// ============================================================================
// Purchase Requisitions (Oracle Fusion Self-Service Procurement > Requisitions)
// ============================================================================

/// Purchase Requisition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRequisition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_number: String,
    pub description: Option<String>,
    pub urgency_code: String,
    pub status: String,
    pub requester_id: Option<Uuid>,
    pub requester_name: Option<String>,
    pub department: Option<String>,
    pub justification: Option<String>,
    pub budget_code: Option<String>,
    pub amount_limit: Option<String>,
    pub total_amount: String,
    pub currency_code: String,
    pub charge_account_code: Option<String>,
    pub delivery_address: Option<String>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub notes: Option<String>,
    pub lines: Vec<RequisitionLine>,
    pub approvals: Vec<RequisitionApproval>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Requisition Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_id: Uuid,
    pub line_number: i32,
    pub item_code: Option<String>,
    pub item_description: String,
    pub category: Option<String>,
    pub quantity: String,
    pub unit_of_measure: String,
    pub unit_price: String,
    pub line_amount: String,
    pub currency_code: String,
    pub charge_account_code: Option<String>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub status: String,
    pub source_type: String,
    pub source_reference: Option<String>,
    pub notes: Option<String>,
    pub distributions: Vec<RequisitionDistribution>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Requisition Distribution (accounting split)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionDistribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_id: Uuid,
    pub line_id: Uuid,
    pub distribution_number: i32,
    pub charge_account_code: String,
    pub allocation_percentage: String,
    pub amount: String,
    pub project_code: Option<String>,
    pub cost_center: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Requisition Approval
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionApproval {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_id: Uuid,
    pub approver_id: Uuid,
    pub approver_name: Option<String>,
    pub action: String,
    pub comments: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// AutoCreate Link (requisition to PO tracking)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutocreateLink {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_id: Uuid,
    pub requisition_line_id: Uuid,
    pub purchase_order_id: Option<Uuid>,
    pub purchase_order_number: Option<String>,
    pub purchase_order_line_id: Option<Uuid>,
    pub purchase_order_line_number: Option<i32>,
    pub quantity_ordered: String,
    pub status: String,
    pub autocreate_date: chrono::DateTime<chrono::Utc>,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Purchase Requisition Create Request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseRequisitionRequest {
    pub description: Option<String>,
    pub urgency_code: Option<String>,
    pub requester_id: Option<Uuid>,
    pub requester_name: Option<String>,
    pub department: Option<String>,
    pub justification: Option<String>,
    pub budget_code: Option<String>,
    pub amount_limit: Option<String>,
    pub currency_code: Option<String>,
    pub charge_account_code: Option<String>,
    pub delivery_address: Option<String>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub notes: Option<String>,
    pub lines: Vec<RequisitionLineRequest>,
}

/// Requisition Line Create Request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionLineRequest {
    pub item_code: Option<String>,
    pub item_description: String,
    pub category: Option<String>,
    pub quantity: Option<String>,
    pub unit_of_measure: Option<String>,
    pub unit_price: Option<String>,
    pub currency_code: Option<String>,
    pub charge_account_code: Option<String>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub source_type: Option<String>,
    pub source_reference: Option<String>,
    pub notes: Option<String>,
    pub distributions: Option<Vec<RequisitionDistributionRequest>>,
}

/// Requisition Distribution Create Request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionDistributionRequest {
    pub charge_account_code: String,
    pub allocation_percentage: Option<String>,
    pub amount: Option<String>,
    pub project_code: Option<String>,
    pub cost_center: Option<String>,
}

/// AutoCreate Request (convert requisitions to POs)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutocreateRequest {
    pub requisition_line_ids: Vec<Uuid>,
    pub purchase_order_number: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
}

/// Requisition Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequisitionDashboardSummary {
    pub total_requisitions: i32,
    pub draft_requisitions: i32,
    pub submitted_requisitions: i32,
    pub approved_requisitions: i32,
    pub rejected_requisitions: i32,
    pub cancelled_requisitions: i32,
    pub total_amount: String,
    pub autocreate_pending: i32,
    pub autocreate_ordered: i32,
    pub by_priority: serde_json::Value,
}

// ============================================================================
// Benefits Administration (Oracle Fusion HCM > Benefits)
// ============================================================================

/// Benefits plan definition
/// Oracle Fusion: Benefits > Benefits Plans
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenefitsPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Plan type: medical, dental, vision, life_insurance, disability, retirement, hsa, fsa
    pub plan_type: String,
    /// Coverage tier options available for this plan
    pub coverage_tiers: serde_json::Value,
    /// Provider/insurance carrier name
    pub provider_name: Option<String>,
    /// External plan ID from carrier
    pub provider_plan_id: Option<String>,
    /// Plan year start date
    pub plan_year_start: Option<chrono::NaiveDate>,
    /// Plan year end date
    pub plan_year_end: Option<chrono::NaiveDate>,
    /// Open enrollment start date
    pub open_enrollment_start: Option<chrono::NaiveDate>,
    /// Open enrollment end date
    pub open_enrollment_end: Option<chrono::NaiveDate>,
    /// Whether employees can make mid-year changes (qualifying life events)
    pub allow_life_event_changes: bool,
    /// Whether the plan requires evidence of insurability
    pub requires_eoi: bool,
    /// Waiting period in days before new hires can enroll
    pub waiting_period_days: i32,
    /// Maximum number of dependents allowed
    pub max_dependents: Option<i32>,
    /// Whether the plan is currently active and available for enrollment
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Benefits plan coverage tier (e.g., Employee Only, Employee + Spouse, Family)
/// Oracle Fusion: Benefits > Coverage Options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageTier {
    /// Tier code: employee_only, employee_spouse, employee_child, family
    pub tier_code: String,
    /// Human-readable tier name
    pub tier_name: String,
    /// Employee's contribution per pay period (employer-paid portion excluded)
    pub employee_cost: String,
    /// Employer's contribution per pay period
    pub employer_cost: String,
    /// Total cost per pay period
    pub total_cost: String,
}

/// Employee benefits enrollment
/// Oracle Fusion: Benefits > Enrollments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenefitsEnrollment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub plan_id: Uuid,
    pub plan_code: Option<String>,
    pub plan_name: Option<String>,
    pub plan_type: Option<String>,
    /// Selected coverage tier code
    pub coverage_tier: String,
    /// Enrollment type: open_enrollment, new_hire, life_event, manual
    pub enrollment_type: String,
    /// Enrollment status: pending, active, waived, cancelled, suspended
    pub status: String,
    /// Effective start date of coverage
    pub effective_start_date: chrono::NaiveDate,
    /// Effective end date of coverage
    pub effective_end_date: Option<chrono::NaiveDate>,
    /// Employee cost per pay period (deduction amount)
    pub employee_cost: String,
    /// Employer cost per pay period
    pub employer_cost: String,
    /// Total cost per pay period
    pub total_cost: String,
    /// Payroll deduction frequency: per_pay_period, monthly, semi_monthly
    pub deduction_frequency: String,
    /// GL account code for employee deduction
    pub deduction_account_code: Option<String>,
    /// GL account code for employer contribution
    pub employer_contribution_account_code: Option<String>,
    /// Enrolled dependents
    pub dependents: serde_json::Value,
    /// Qualifying life event reason (if enrollment due to life event)
    pub life_event_reason: Option<String>,
    /// Life event date
    pub life_event_date: Option<chrono::NaiveDate>,
    /// Who processed this enrollment
    pub processed_by: Option<Uuid>,
    /// When the enrollment was processed
    pub processed_at: Option<DateTime<Utc>>,
    /// Cancellation reason
    pub cancellation_reason: Option<String>,
    /// When the enrollment was cancelled
    pub cancelled_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Benefits deduction record (tracks each payroll deduction)
/// Oracle Fusion: Payroll > Element Entries > Benefits Deductions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenefitsDeduction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub enrollment_id: Uuid,
    pub employee_id: Uuid,
    pub plan_id: Uuid,
    pub plan_code: Option<String>,
    pub plan_name: Option<String>,
    /// Deduction amount (employee portion)
    pub employee_amount: String,
    /// Employer contribution amount
    pub employer_amount: String,
    /// Total deduction amount
    pub total_amount: String,
    /// Pay period this deduction applies to
    pub pay_period_start: chrono::NaiveDate,
    pub pay_period_end: chrono::NaiveDate,
    /// GL account code for the deduction
    pub deduction_account_code: Option<String>,
    /// Whether the deduction has been processed through payroll
    pub is_processed: bool,
    pub processed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Benefits enrollment summary (dashboard view)
/// Oracle Fusion: Benefits > Benefits Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenefitsSummary {
    pub total_plans: i32,
    pub active_plans: i32,
    pub total_enrollments: i32,
    pub active_enrollments: i32,
    pub pending_enrollments: i32,
    pub waived_enrollments: i32,
    pub total_employee_cost: String,
    pub total_employer_cost: String,
    pub enrollments_by_plan_type: serde_json::Value,
}

// ============================================================================
// AutoInvoice (Oracle Fusion Receivables AutoInvoice)
// ============================================================================

/// AutoInvoice grouping rule definition.
/// Oracle Fusion: Receivables > AutoInvoice > Grouping Rules
/// Controls how imported transaction lines are grouped into invoices.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceGroupingRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// Transaction types this rule applies to (e.g., ["invoice", "credit_memo"])
    pub transaction_types: serde_json::Value,
    /// Fields to group by (e.g., ["bill_to_customer_id", "currency_code"])
    pub group_by_fields: serde_json::Value,
    /// Line ordering fields (e.g., ["line_number", "item_code"])
    pub line_order_by: serde_json::Value,
    /// Whether this is the default grouping rule
    pub is_default: bool,
    pub is_active: bool,
    pub priority: i32,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// AutoInvoice validation rule.
/// Oracle Fusion: Receivables > AutoInvoice > Validation Rules
/// Validates transaction lines before invoice creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceValidationRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// The field being validated
    pub field_name: String,
    /// Validation type: "required", "format", "reference", "range", "custom"
    pub validation_type: String,
    /// Expression or value for the validation
    pub validation_expression: Option<String>,
    /// Error message when validation fails
    pub error_message: String,
    /// Whether to reject the entire line on failure (vs. flag as warning)
    pub is_fatal: bool,
    /// Transaction types this rule applies to
    pub transaction_types: serde_json::Value,
    pub is_active: bool,
    pub priority: i32,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// AutoInvoice import batch (header for a batch of transaction lines being imported).
/// Oracle Fusion: Receivables > AutoInvoice > Import
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub batch_source: String,
    pub description: Option<String>,
    /// 'pending', 'validating', 'validated', 'processing', 'completed', 'failed', 'cancelled'
    pub status: String,
    pub total_lines: i32,
    pub valid_lines: i32,
    pub invalid_lines: i32,
    pub invoices_created: i32,
    pub invoices_total_amount: String,
    pub grouping_rule_id: Option<Uuid>,
    pub validation_errors: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// AutoInvoice transaction line (raw line being imported).
/// Oracle Fusion: Receivables > AutoInvoice > Interface Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub line_number: i32,
    /// Source system identifier
    pub source_line_id: Option<String>,
    /// Transaction type: "invoice", "credit_memo", "debit_memo", "on_account_credit"
    pub transaction_type: String,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub bill_to_customer_id: Option<Uuid>,
    pub bill_to_site_id: Option<Uuid>,
    pub ship_to_customer_id: Option<Uuid>,
    pub ship_to_site_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity: Option<String>,
    pub unit_of_measure: Option<String>,
    pub unit_price: String,
    pub line_amount: String,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub revenue_account_code: Option<String>,
    pub receivable_account_code: Option<String>,
    pub tax_code: Option<String>,
    pub tax_amount: Option<String>,
    pub sales_rep_id: Option<Uuid>,
    pub sales_rep_name: Option<String>,
    pub memo_line: Option<String>,
    pub reference_number: Option<String>,
    pub sales_order_number: Option<String>,
    pub sales_order_line: Option<String>,
    /// 'pending', 'valid', 'invalid', 'grouped', 'error'
    pub status: String,
    pub validation_errors: serde_json::Value,
    /// Assigned invoice ID after grouping
    pub invoice_id: Option<Uuid>,
    pub invoice_line_number: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// AutoInvoice result — the generated AR invoice.
/// Oracle Fusion: Receivables > Transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceResult {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_id: Uuid,
    pub invoice_number: String,
    pub transaction_type: String,
    pub customer_id: Option<Uuid>,
    pub bill_to_customer_id: Option<Uuid>,
    pub bill_to_site_id: Option<Uuid>,
    pub ship_to_customer_id: Option<Uuid>,
    pub ship_to_site_id: Option<Uuid>,
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub transaction_date: chrono::NaiveDate,
    pub gl_date: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub subtotal: String,
    pub tax_amount: String,
    pub total_amount: String,
    pub line_count: i32,
    pub receivable_account_code: Option<String>,
    pub sales_rep_id: Option<Uuid>,
    pub sales_order_number: Option<String>,
    pub reference_number: Option<String>,
    /// 'draft', 'complete', 'posted', 'cancelled'
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// AutoInvoice result line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceResultLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub invoice_id: Uuid,
    pub line_number: i32,
    pub source_line_id: Option<String>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity: Option<String>,
    pub unit_of_measure: Option<String>,
    pub unit_price: String,
    pub line_amount: String,
    pub tax_code: Option<String>,
    pub tax_amount: Option<String>,
    pub revenue_account_code: Option<String>,
    pub sales_order_number: Option<String>,
    pub sales_order_line: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// AutoInvoice import request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceImportRequest {
    pub batch_source: String,
    pub description: Option<String>,
    pub lines: Vec<AutoInvoiceLineRequest>,
    pub grouping_rule_id: Option<Uuid>,
}

/// Single line in an AutoInvoice import
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceLineRequest {
    pub source_line_id: Option<String>,
    pub transaction_type: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_number: Option<String>,
    pub customer_name: Option<String>,
    pub bill_to_customer_id: Option<Uuid>,
    pub bill_to_site_id: Option<Uuid>,
    pub ship_to_customer_id: Option<Uuid>,
    pub ship_to_site_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity: Option<String>,
    pub unit_of_measure: Option<String>,
    pub unit_price: Option<String>,
    pub line_amount: Option<String>,
    #[serde(default = "default_currency_usd")]
    pub currency_code: String,
    pub exchange_rate: Option<String>,
    pub transaction_date: Option<chrono::NaiveDate>,
    pub gl_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub revenue_account_code: Option<String>,
    pub receivable_account_code: Option<String>,
    pub tax_code: Option<String>,
    pub tax_amount: Option<String>,
    pub sales_rep_id: Option<Uuid>,
    pub sales_rep_name: Option<String>,
    pub memo_line: Option<String>,
    pub reference_number: Option<String>,
    pub sales_order_number: Option<String>,
    pub sales_order_line: Option<String>,
}

/// AutoInvoice processing summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceSummary {
    pub total_batches: i32,
    pub pending_batches: i32,
    pub completed_batches: i32,
    pub failed_batches: i32,
    pub total_lines_imported: i32,
    pub total_invoices_created: i32,
    pub total_invoice_amount: String,
}

/// Validation error for an AutoInvoice line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoInvoiceValidationError {
    pub line_number: i32,
    pub field_name: String,
    pub validation_rule: String,
    pub error_message: String,
    pub is_fatal: bool,
}

// ============================================================================
// Performance Management (Oracle Fusion HCM Performance Review)
// ============================================================================

/// Performance rating model (defines the rating scale).
/// Oracle Fusion: My Client Groups > Performance > Rating Models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceRatingModel {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Rating scale entries, e.g. [{"value":1,"label":"Below Expectations"}, ...]
    pub rating_scale: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance review cycle (defines a review period).
/// Oracle Fusion: My Client Groups > Performance > Review Cycles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceReviewCycle {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// Cycle type: annual, mid_year, quarterly, project_end, probation
    pub cycle_type: String,
    /// Status: draft, planning, goal_setting, self_evaluation, manager_evaluation, calibration, completed, cancelled
    pub status: String,
    pub rating_model_id: Option<Uuid>,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub goal_setting_start: Option<chrono::NaiveDate>,
    pub goal_setting_end: Option<chrono::NaiveDate>,
    pub self_evaluation_start: Option<chrono::NaiveDate>,
    pub self_evaluation_end: Option<chrono::NaiveDate>,
    pub manager_evaluation_start: Option<chrono::NaiveDate>,
    pub manager_evaluation_end: Option<chrono::NaiveDate>,
    pub calibration_date: Option<chrono::NaiveDate>,
    pub require_goals: bool,
    pub require_competencies: bool,
    pub min_goals: i32,
    pub max_goals: i32,
    pub goal_weight_total: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Competency definition.
/// Oracle Fusion: My Client Groups > Performance > Competencies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceCompetency {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// Category: core, leadership, technical, functional
    pub category: Option<String>,
    pub rating_model_id: Option<Uuid>,
    pub behavioral_indicators: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance document (one per employee per review cycle).
/// Oracle Fusion: My Client Groups > Performance > Performance Documents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceDocument {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub review_cycle_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
    pub document_number: String,
    /// Status: not_started, goal_setting, self_evaluation, manager_evaluation, calibration, completed, cancelled
    pub status: String,
    pub overall_rating: Option<String>,
    pub overall_rating_label: Option<String>,
    pub self_overall_rating: Option<String>,
    pub self_comments: Option<String>,
    pub manager_overall_rating: Option<String>,
    pub manager_comments: Option<String>,
    pub calibration_rating: Option<String>,
    pub calibration_comments: Option<String>,
    pub final_rating: Option<String>,
    pub final_comments: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance goal (linked to a performance document).
/// Oracle Fusion: My Client Groups > Performance > Goals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceGoal {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub document_id: Uuid,
    pub employee_id: Uuid,
    pub goal_name: String,
    pub description: Option<String>,
    /// Category: performance, development, project, behavioral
    pub goal_category: Option<String>,
    /// Status: draft, active, completed, cancelled
    pub status: String,
    pub weight: String,
    pub target_metric: Option<String>,
    pub actual_result: Option<String>,
    pub self_rating: Option<String>,
    pub self_comments: Option<String>,
    pub manager_rating: Option<String>,
    pub manager_comments: Option<String>,
    pub start_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub completed_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Competency assessment (linked to a performance document).
/// Oracle Fusion: My Client Groups > Performance > Competency Assessments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompetencyAssessment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub document_id: Uuid,
    pub employee_id: Uuid,
    pub competency_id: Uuid,
    pub self_rating: Option<String>,
    pub self_comments: Option<String>,
    pub manager_rating: Option<String>,
    pub manager_comments: Option<String>,
    pub calibration_rating: Option<String>,
    pub calibration_comments: Option<String>,
    pub final_rating: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance feedback (360-degree feedback, ad-hoc).
/// Oracle Fusion: My Client Groups > Performance > Feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceFeedback {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub document_id: Option<Uuid>,
    pub employee_id: Uuid,
    pub from_user_id: Uuid,
    pub from_user_name: Option<String>,
    /// Feedback type: peer, manager, direct_report, external, self
    pub feedback_type: String,
    pub subject: Option<String>,
    pub content: String,
    /// Visibility: private, manager_only, manager_and_employee, everyone
    pub visibility: String,
    /// Status: draft, submitted, acknowledged, withdrawn
    pub status: String,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Performance dashboard summary for a review cycle.
/// Oracle Fusion: My Client Groups > Performance > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceDashboard {
    pub review_cycle_id: Uuid,
    pub total_documents: i32,
    pub not_started_count: i32,
    pub goal_setting_count: i32,
    pub self_evaluation_count: i32,
    pub manager_evaluation_count: i32,
    pub calibration_count: i32,
    pub completed_count: i32,
    pub cancelled_count: i32,
    pub average_rating: Option<String>,
    pub goals_total: i32,
    pub goals_completed: i32,
    pub feedback_count: i32,
}

// ============================================================================
// Credit Management Types
// Oracle Fusion Cloud: Receivables > Credit Management
// ============================================================================

/// Credit scoring model defines how customer creditworthiness is assessed.
/// Oracle Fusion: Credit Management > Credit Scoring Models
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditScoringModel {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub model_type: String,
    pub scoring_criteria: serde_json::Value,
    pub score_ranges: serde_json::Value,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit profile per customer or customer group.
/// Oracle Fusion: Credit Management > Credit Profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditProfile {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub profile_number: String,
    pub profile_name: String,
    pub description: Option<String>,
    pub profile_type: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub customer_group_id: Option<Uuid>,
    pub customer_group_name: Option<String>,
    pub scoring_model_id: Option<Uuid>,
    pub credit_score: Option<String>,
    pub credit_rating: Option<String>,
    pub risk_level: String,
    pub status: String,
    pub review_frequency_days: i32,
    pub last_review_date: Option<chrono::NaiveDate>,
    pub next_review_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit limit (supports multi-currency and global limits).
/// Oracle Fusion: Credit Management > Credit Limits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditLimit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub profile_id: Uuid,
    pub limit_type: String,
    pub currency_code: Option<String>,
    pub credit_limit: String,
    pub temp_limit_increase: String,
    pub temp_limit_expiry: Option<chrono::NaiveDate>,
    pub used_amount: String,
    pub available_amount: String,
    pub hold_amount: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit check rule defines when credit checks are triggered.
/// Oracle Fusion: Credit Management > Credit Check Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditCheckRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub check_point: String,
    pub check_type: String,
    pub condition: serde_json::Value,
    pub action_on_failure: String,
    pub priority: i32,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit exposure tracks total exposure per profile.
/// Oracle Fusion: Credit Management > Credit Exposure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditExposure {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub profile_id: Uuid,
    pub exposure_date: chrono::NaiveDate,
    pub open_receivables: String,
    pub open_orders: String,
    pub open_shipments: String,
    pub open_invoices: String,
    pub unapplied_cash: String,
    pub on_hold_amount: String,
    pub total_exposure: String,
    pub credit_limit: String,
    pub available_credit: String,
    pub utilization_percent: String,
    pub currency_code: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit hold placed on transactions when credit limits exceeded.
/// Oracle Fusion: Credit Management > Credit Holds
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditHold {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub profile_id: Uuid,
    pub hold_number: String,
    pub hold_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_number: Option<String>,
    pub hold_amount: Option<String>,
    pub reason: Option<String>,
    pub status: String,
    pub released_by: Option<Uuid>,
    pub released_at: Option<DateTime<Utc>>,
    pub release_reason: Option<String>,
    pub overridden_by: Option<Uuid>,
    pub overridden_at: Option<DateTime<Utc>>,
    pub override_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit review (periodic or triggered review of a credit profile).
/// Oracle Fusion: Credit Management > Credit Reviews
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditReview {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub profile_id: Uuid,
    pub review_number: String,
    pub review_type: String,
    pub status: String,
    pub previous_credit_limit: Option<String>,
    pub recommended_credit_limit: Option<String>,
    pub approved_credit_limit: Option<String>,
    pub previous_score: Option<String>,
    pub new_score: Option<String>,
    pub previous_rating: Option<String>,
    pub new_rating: Option<String>,
    pub findings: Option<String>,
    pub recommendations: Option<String>,
    pub reviewer_id: Option<Uuid>,
    pub reviewer_name: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub approver_id: Option<Uuid>,
    pub approver_name: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credit management dashboard summary.
/// Oracle Fusion: Credit Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditManagementDashboard {
    pub total_profiles: i32,
    pub active_profiles: i32,
    pub blocked_profiles: i32,
    pub total_credit_limit: String,
    pub total_exposure: String,
    pub total_available: String,
    pub active_holds: i32,
    pub pending_reviews: i32,
    pub overdue_reviews: i32,
    pub average_utilization: String,
}

// ============================================================================
// Product Information Management (PIM)
// Oracle Fusion Cloud: Product Hub / Product Information Management
//
// Central product master data management including items, categories,
// cross-references, lifecycle phases, and new item request workflows.
// ============================================================================

/// Item status within its lifecycle.
/// Oracle Fusion: Product Development > Item > Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatus {
    Draft,
    Active,
    Obsolete,
    Inactive,
    PendingApproval,
}

impl std::fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemStatus::Draft => write!(f, "draft"),
            ItemStatus::Active => write!(f, "active"),
            ItemStatus::Obsolete => write!(f, "obsolete"),
            ItemStatus::Inactive => write!(f, "inactive"),
            ItemStatus::PendingApproval => write!(f, "pending_approval"),
        }
    }
}

/// Lifecycle phase for an item.
/// Oracle Fusion: Product Hub > Item Lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LifecyclePhase {
    Concept,
    Design,
    Prototype,
    Production,
    PhaseOut,
    Obsolete,
}

impl std::fmt::Display for LifecyclePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecyclePhase::Concept => write!(f, "concept"),
            LifecyclePhase::Design => write!(f, "design"),
            LifecyclePhase::Prototype => write!(f, "prototype"),
            LifecyclePhase::Production => write!(f, "production"),
            LifecyclePhase::PhaseOut => write!(f, "phase_out"),
            LifecyclePhase::Obsolete => write!(f, "obsolete"),
        }
    }
}

/// Product item master record.
/// Oracle Fusion: Product Hub > Manage Items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_number: String,
    pub item_name: String,
    pub description: Option<String>,
    pub long_description: Option<String>,
    pub item_type: String, // finished_good, subassembly, component, service, supply, expense
    pub status: String,
    pub lifecycle_phase: String,
    pub primary_uom_code: String,
    pub secondary_uom_code: Option<String>,
    pub weight: Option<String>,
    pub weight_uom: Option<String>,
    pub volume: Option<String>,
    pub volume_uom: Option<String>,
    pub hazmat_flag: bool,
    pub lot_control_flag: bool,
    pub serial_control_flag: bool,
    pub shelf_life_days: Option<i32>,
    pub min_order_quantity: Option<String>,
    pub max_order_quantity: Option<String>,
    pub lead_time_days: Option<i32>,
    pub list_price: Option<String>,
    pub cost_price: Option<String>,
    pub currency_code: String,
    pub inventory_item_flag: bool,
    pub purchasable_flag: bool,
    pub sellable_flag: bool,
    pub stock_enabled_flag: bool,
    pub invoice_enabled_flag: bool,
    pub default_buyer_id: Option<Uuid>,
    pub default_supplier_id: Option<Uuid>,
    pub template_id: Option<Uuid>,
    pub thumbnail_url: Option<String>,
    pub image_url: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// PIM Item category for hierarchical classification.
/// Oracle Fusion: Product Hub > Manage Item Classes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub level_number: i32,
    pub item_count: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// PIM item-to-category assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimCategoryAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_id: Uuid,
    pub category_id: Uuid,
    pub is_primary: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Item cross-reference for mapping item identifiers across systems.
/// Oracle Fusion: Product Hub > Manage Cross-References
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimCrossReference {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_id: Uuid,
    pub cross_reference_type: String, // gtin, upc, ean, supplier, customer, internal, other
    pub cross_reference_value: String,
    pub description: Option<String>,
    pub source_system: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New Item Request (NIR) for introducing new products through workflow.
/// Oracle Fusion: Product Hub > New Item Request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimNewItemRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_number: String,
    pub title: String,
    pub description: Option<String>,
    pub item_type: String,
    pub priority: String, // low, medium, high, critical
    pub status: String,   // draft, submitted, approved, rejected, implemented, cancelled
    pub requested_item_number: Option<String>,
    pub requested_item_name: Option<String>,
    pub requested_category_id: Option<Uuid>,
    pub justification: Option<String>,
    pub target_launch_date: Option<chrono::NaiveDate>,
    pub estimated_cost: Option<String>,
    pub currency_code: String,
    pub requested_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejection_reason: Option<String>,
    pub implemented_item_id: Option<Uuid>,
    pub implemented_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Item template for standardizing item creation.
/// Oracle Fusion: Product Hub > Manage Item Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimItemTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub item_type: String,
    pub default_uom_code: Option<String>,
    pub default_category_id: Option<Uuid>,
    pub default_inventory_flag: bool,
    pub default_purchasable_flag: bool,
    pub default_sellable_flag: bool,
    pub default_stock_enabled_flag: bool,
    pub attribute_defaults: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// PIM Dashboard summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PimDashboard {
    pub total_items: i32,
    pub active_items: i32,
    pub draft_items: i32,
    pub obsolete_items: i32,
    pub total_categories: i32,
    pub pending_nir_count: i32,
    pub approved_nir_count: i32,
    pub cross_reference_count: i32,
    pub recently_created_items: i32,
    pub items_by_type: serde_json::Value,
}

// ============================================================================
// Quality Management
// Oracle Fusion Cloud: Quality Management
//
// Quality inspection plans, inspections, non-conformance reports (NCRs),
// corrective & preventive actions (CAPA), quality holds, and dashboards.
// ============================================================================

/// Quality Inspection Plan - defines what to inspect, how, and when
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInspectionPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_code: String,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub inspection_trigger: String,
    pub sampling_method: String,
    pub sample_size_percent: String,
    pub accept_number: Option<i32>,
    pub reject_number: Option<i32>,
    pub frequency: String,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub total_criteria: i32,
    pub total_inspections: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Quality Inspection Plan Criterion - a specific check within a plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInspectionPlanCriterion {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Uuid,
    pub criterion_number: i32,
    pub name: String,
    pub description: Option<String>,
    pub characteristic: String,
    pub measurement_type: String,
    pub target_value: String,
    pub lower_spec_limit: String,
    pub upper_spec_limit: String,
    pub unit_of_measure: Option<String>,
    pub is_mandatory: bool,
    pub weight: String,
    pub criticality: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Quality Inspection - an instance of executing an inspection plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInspection {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inspection_number: String,
    pub plan_id: Uuid,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub lot_number: Option<String>,
    pub quantity_inspected: String,
    pub quantity_accepted: String,
    pub quantity_rejected: String,
    pub unit_of_measure: Option<String>,
    pub status: String,
    pub verdict: String,
    pub overall_score: String,
    pub notes: Option<String>,
    pub inspector_id: Option<Uuid>,
    pub inspector_name: Option<String>,
    pub inspection_date: chrono::NaiveDate,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Quality Inspection Result - individual criterion result within an inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityInspectionResult {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inspection_id: Uuid,
    pub criterion_id: Option<Uuid>,
    pub criterion_name: String,
    pub characteristic: String,
    pub measurement_type: String,
    pub observed_value: String,
    pub target_value: String,
    pub lower_spec_limit: String,
    pub upper_spec_limit: String,
    pub unit_of_measure: Option<String>,
    pub result_status: String,
    pub deviation: String,
    pub notes: Option<String>,
    pub evaluated_by: Option<Uuid>,
    pub evaluated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Non-Conformance Report (NCR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonConformanceReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ncr_number: String,
    pub title: String,
    pub description: Option<String>,
    pub ncr_type: String,
    pub severity: String,
    pub origin: String,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub detected_date: chrono::NaiveDate,
    pub detected_by: Option<String>,
    pub responsible_party: Option<String>,
    pub status: String,
    pub resolution_description: Option<String>,
    pub resolution_type: Option<String>,
    pub resolved_by: Option<String>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub total_corrective_actions: i32,
    pub open_corrective_actions: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Corrective & Preventive Action (CAPA)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectiveAction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub ncr_id: Uuid,
    pub action_number: String,
    pub action_type: String,
    pub title: String,
    pub description: Option<String>,
    pub root_cause: Option<String>,
    pub corrective_action_desc: Option<String>,
    pub preventive_action_desc: Option<String>,
    pub assigned_to: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub effectiveness_rating: Option<i32>,
    pub priority: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Quality Hold - prevents items/lots from being used until resolved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityHold {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub hold_number: String,
    pub reason: String,
    pub description: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub lot_number: Option<String>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub hold_type: String,
    pub status: String,
    pub released_by: Option<Uuid>,
    pub released_at: Option<chrono::DateTime<chrono::Utc>>,
    pub release_notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Quality Management Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDashboardSummary {
    pub total_active_plans: i32,
    pub total_pending_inspections: i32,
    pub total_passed_inspections: i32,
    pub total_failed_inspections: i32,
    pub inspection_pass_rate_percent: String,
    pub total_open_ncrs: i32,
    pub total_ncrs: i32,
    pub critical_ncrs: i32,
    pub total_open_corrective_actions: i32,
    pub total_completed_corrective_actions: i32,
    pub corrective_action_completion_rate_percent: String,
    pub total_active_holds: i32,
    pub inspections_by_verdict: serde_json::Value,
    pub ncrs_by_severity: serde_json::Value,
    pub ncrs_by_type: serde_json::Value,
}

// ============================================================================
// Transfer Pricing (Oracle Fusion Financials > Transfer Pricing)
// ============================================================================

/// Transfer Pricing Policy
/// Defines the pricing method and parameters for intercompany transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPricingPolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub policy_code: String,
    pub name: String,
    pub description: Option<String>,
    pub pricing_method: String,
    pub from_entity_id: Option<Uuid>,
    pub from_entity_name: Option<String>,
    pub to_entity_id: Option<Uuid>,
    pub to_entity_name: Option<String>,
    pub product_category: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub geography: Option<String>,
    pub tax_jurisdiction: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub arm_length_range_low: String,
    pub arm_length_range_mid: String,
    pub arm_length_range_high: String,
    pub margin_pct: String,
    pub cost_base: Option<String>,
    pub status: String,
    pub version: i32,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Transfer Price Transaction
/// Individual intercompany transaction with calculated transfer price.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPriceTransaction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub transaction_number: String,
    pub policy_id: Option<Uuid>,
    pub from_entity_id: Option<Uuid>,
    pub from_entity_name: Option<String>,
    pub to_entity_id: Option<Uuid>,
    pub to_entity_name: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity: String,
    pub unit_cost: String,
    pub transfer_price: String,
    pub total_amount: String,
    pub currency_code: String,
    pub transaction_date: chrono::NaiveDate,
    pub gl_date: Option<chrono::NaiveDate>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub margin_applied: Option<String>,
    pub margin_amount: Option<String>,
    pub is_arm_length_compliant: Option<bool>,
    pub compliance_notes: Option<String>,
    pub status: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Benchmark Study (Arm's-Length Analysis)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkStudy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub study_number: String,
    pub title: String,
    pub description: Option<String>,
    pub policy_id: Option<Uuid>,
    pub analysis_method: String,
    pub fiscal_year: Option<i32>,
    pub from_entity_id: Option<Uuid>,
    pub from_entity_name: Option<String>,
    pub to_entity_id: Option<Uuid>,
    pub to_entity_name: Option<String>,
    pub product_category: Option<String>,
    pub tested_party: Option<String>,
    pub interquartile_range_low: String,
    pub interquartile_range_mid: String,
    pub interquartile_range_high: String,
    pub tested_result: String,
    pub is_within_range: Option<bool>,
    pub conclusion: Option<String>,
    pub prepared_by: Option<Uuid>,
    pub prepared_by_name: Option<String>,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_by_name: Option<String>,
    pub status: String,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Benchmark Comparable Company
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkComparable {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub benchmark_id: Uuid,
    pub comparable_number: i32,
    pub company_name: String,
    pub country: Option<String>,
    pub industry_code: Option<String>,
    pub industry_description: Option<String>,
    pub fiscal_year: Option<i32>,
    pub revenue: String,
    pub operating_income: String,
    pub operating_margin_pct: String,
    pub net_income: String,
    pub total_assets: String,
    pub employees: Option<i32>,
    pub data_source: Option<String>,
    pub is_included: bool,
    pub exclusion_reason: Option<String>,
    pub relevance_score: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Documentation Package (BEPS / Local File / Master File / CbCR)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPricingDocumentation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub doc_number: String,
    pub title: String,
    pub doc_type: String,
    pub fiscal_year: i32,
    pub country: Option<String>,
    pub reporting_entity_id: Option<Uuid>,
    pub reporting_entity_name: Option<String>,
    pub description: Option<String>,
    pub content_summary: Option<String>,
    pub policy_ids: Option<serde_json::Value>,
    pub benchmark_ids: Option<serde_json::Value>,
    pub filing_date: Option<chrono::NaiveDate>,
    pub filing_deadline: Option<chrono::NaiveDate>,
    pub responsible_party: Option<String>,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub filed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Transfer Pricing Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferPricingDashboard {
    pub total_policies: i32,
    pub active_policies: i32,
    pub total_transactions: i32,
    pub total_transaction_value: String,
    pub pending_transactions: i32,
    pub non_compliant_transactions: i32,
    pub compliance_rate_pct: String,
    pub total_benchmarks: i32,
    pub active_benchmarks: i32,
    pub benchmarks_within_range: i32,
    pub total_documentation: i32,
    pub pending_filings: i32,
    pub overdue_filings: i32,
    pub transactions_by_method: serde_json::Value,
    pub transactions_by_status: serde_json::Value,
}

// ============================================================================
// Order Management (Oracle Fusion SCM > Order Management)
// ============================================================================

/// Sales Order Header
/// Oracle Fusion equivalent: Order Management > Sales Orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrder {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub order_number: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub customer_po_number: Option<String>,
    pub order_date: chrono::NaiveDate,
    pub requested_ship_date: Option<chrono::NaiveDate>,
    pub actual_ship_date: Option<chrono::NaiveDate>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub actual_delivery_date: Option<chrono::NaiveDate>,
    pub ship_to_address: Option<String>,
    pub bill_to_address: Option<String>,
    pub currency_code: String,
    pub subtotal_amount: String,
    pub tax_amount: String,
    pub shipping_charges: String,
    pub total_amount: String,
    pub payment_terms: Option<String>,
    pub shipping_method: Option<String>,
    pub sales_channel: Option<String>,
    pub salesperson_id: Option<Uuid>,
    pub salesperson_name: Option<String>,
    pub status: String,
    pub fulfillment_status: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub created_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sales Order Line
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesOrderLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub order_id: Uuid,
    pub line_number: i32,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity_ordered: String,
    pub quantity_shipped: String,
    pub quantity_cancelled: String,
    pub quantity_backordered: String,
    pub unit_selling_price: String,
    pub unit_list_price: Option<String>,
    pub line_amount: String,
    pub discount_percent: Option<String>,
    pub discount_amount: Option<String>,
    pub tax_code: Option<String>,
    pub tax_amount: String,
    pub requested_ship_date: Option<chrono::NaiveDate>,
    pub actual_ship_date: Option<chrono::NaiveDate>,
    pub promised_delivery_date: Option<chrono::NaiveDate>,
    pub ship_from_warehouse: Option<String>,
    pub fulfillment_status: String,
    pub status: String,
    pub cancellation_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Order Hold
/// Oracle Fusion: Order Management > Order Holds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderHold {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub order_id: Uuid,
    pub order_line_id: Option<Uuid>,
    pub hold_type: String,
    pub hold_reason: String,
    pub applied_by: Option<Uuid>,
    pub applied_by_name: Option<String>,
    pub released_by: Option<Uuid>,
    pub released_by_name: Option<String>,
    pub released_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Fulfillment Shipment
/// Oracle Fusion: Order Management > Shipments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulfillmentShipment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_number: String,
    pub order_id: Uuid,
    pub order_line_ids: serde_json::Value,
    pub warehouse: Option<String>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub shipping_method: Option<String>,
    pub ship_date: Option<chrono::NaiveDate>,
    pub estimated_delivery_date: Option<chrono::NaiveDate>,
    pub actual_delivery_date: Option<chrono::NaiveDate>,
    pub delivery_confirmation: Option<String>,
    pub status: String,
    pub shipped_by: Option<Uuid>,
    pub shipped_by_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create Sales Order Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSalesOrderRequest {
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub customer_po_number: Option<String>,
    pub order_date: chrono::NaiveDate,
    pub requested_ship_date: Option<chrono::NaiveDate>,
    pub requested_delivery_date: Option<chrono::NaiveDate>,
    pub ship_to_address: Option<String>,
    pub bill_to_address: Option<String>,
    pub currency_code: String,
    pub payment_terms: Option<String>,
    pub shipping_method: Option<String>,
    pub sales_channel: Option<String>,
    pub salesperson_id: Option<Uuid>,
    pub salesperson_name: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Add Order Line Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddOrderLineRequest {
    pub org_id: Uuid,
    pub order_id: Uuid,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity_ordered: String,
    pub unit_selling_price: String,
    pub unit_list_price: Option<String>,
    pub discount_percent: Option<String>,
    pub discount_amount: Option<String>,
    pub tax_code: Option<String>,
    pub requested_ship_date: Option<chrono::NaiveDate>,
    pub promised_delivery_date: Option<chrono::NaiveDate>,
    pub ship_from_warehouse: Option<String>,
}

/// Order Management Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderManagementDashboard {
    pub total_orders: i32,
    pub open_orders: i32,
    pub orders_in_fulfillment: i32,
    pub completed_orders: i32,
    pub cancelled_orders: i32,
    pub total_order_value: String,
    pub average_order_value: String,
    pub orders_on_hold: i32,
    pub backordered_lines: i32,
    pub overdue_shipments: i32,
    pub orders_by_status: serde_json::Value,
    pub orders_by_channel: serde_json::Value,
    pub fulfillment_rate_pct: String,
    pub on_time_shipment_pct: String,
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
    /// 'all', 'by_category', 'by_role', 'by_entity'
    pub delegation_type: String,
    /// For type 'by_category': categories as JSON array
    pub categories: serde_json::Value,
    /// For type 'by_role': roles as JSON array
    pub roles: serde_json::Value,
    /// For type 'by_entity': entity types as JSON array
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

fn default_delegation_all() -> String { "all".to_string() }

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

// ============================================================================
// Manufacturing Execution (Oracle Fusion SCM > Manufacturing)
// ============================================================================

/// Work Definition (BOM + Routing template)
/// Oracle Fusion equivalent: Manufacturing > Work Definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub definition_number: String,
    pub description: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub version: i32,
    pub status: String,
    pub production_type: String,
    pub planning_type: String,
    pub standard_lot_size: String,
    pub unit_of_measure: String,
    pub lead_time_days: i32,
    pub cost_type: String,
    pub standard_cost: String,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Work Definition Component (BOM line)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDefinitionComponent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub work_definition_id: Uuid,
    pub line_number: i32,
    pub component_item_id: Option<Uuid>,
    pub component_item_code: String,
    pub component_item_description: Option<String>,
    pub quantity_required: String,
    pub unit_of_measure: String,
    pub component_type: String,
    pub scrap_percent: String,
    pub yield_percent: String,
    pub supply_type: String,
    pub supply_subinventory: Option<String>,
    pub wip_supply_type: String,
    pub operation_sequence: Option<i32>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Work Definition Operation (Routing step)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDefinitionOperation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub work_definition_id: Uuid,
    pub operation_sequence: i32,
    pub operation_name: String,
    pub operation_description: Option<String>,
    pub work_center_code: Option<String>,
    pub work_center_name: Option<String>,
    pub department_code: Option<String>,
    pub setup_hours: String,
    pub run_time_hours: String,
    pub run_time_unit: String,
    pub units_per_run: String,
    pub resource_code: Option<String>,
    pub resource_type: String,
    pub resource_count: i32,
    pub standard_labor_cost: String,
    pub standard_overhead_cost: String,
    pub standard_machine_cost: String,
    pub operation_type: String,
    pub backflush_enabled: bool,
    pub count_point_type: String,
    pub yield_percent: String,
    pub scrap_percent: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Work Order (Production Order)
/// Oracle Fusion equivalent: Manufacturing > Work Orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOrder {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub work_order_number: String,
    pub description: Option<String>,
    pub work_definition_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity_ordered: String,
    pub quantity_completed: String,
    pub quantity_scrapped: String,
    pub quantity_in_queue: String,
    pub quantity_running: String,
    pub quantity_rejected: String,
    pub unit_of_measure: String,
    pub scheduled_start_date: Option<chrono::NaiveDate>,
    pub scheduled_completion_date: Option<chrono::NaiveDate>,
    pub actual_start_date: Option<chrono::NaiveDate>,
    pub actual_completion_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub priority: String,
    pub production_line: Option<String>,
    pub work_center_code: Option<String>,
    pub warehouse_code: Option<String>,
    pub cost_type: String,
    pub estimated_material_cost: String,
    pub estimated_labor_cost: String,
    pub estimated_overhead_cost: String,
    pub estimated_total_cost: String,
    pub actual_material_cost: String,
    pub actual_labor_cost: String,
    pub actual_overhead_cost: String,
    pub actual_total_cost: String,
    pub source_type: Option<String>,
    pub source_document_number: Option<String>,
    pub source_document_line_id: Option<Uuid>,
    pub firm_planned: bool,
    pub company_id: Option<Uuid>,
    pub plant_code: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub released_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Work Order Operation (tracking per-operation progress)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOrderOperation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub work_order_id: Uuid,
    pub operation_sequence: i32,
    pub operation_name: String,
    pub work_center_code: Option<String>,
    pub work_center_name: Option<String>,
    pub department_code: Option<String>,
    pub quantity_in_queue: String,
    pub quantity_running: String,
    pub quantity_completed: String,
    pub quantity_rejected: String,
    pub quantity_scrapped: String,
    pub scheduled_start_date: Option<chrono::NaiveDate>,
    pub scheduled_completion_date: Option<chrono::NaiveDate>,
    pub actual_start_date: Option<chrono::NaiveDate>,
    pub actual_completion_date: Option<chrono::NaiveDate>,
    pub actual_setup_hours: String,
    pub actual_run_hours: String,
    pub resource_code: Option<String>,
    pub resource_type: String,
    pub status: String,
    pub actual_labor_cost: String,
    pub actual_overhead_cost: String,
    pub actual_machine_cost: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Work Order Material Requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkOrderMaterial {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub work_order_id: Uuid,
    pub operation_sequence: Option<i32>,
    pub component_item_id: Option<Uuid>,
    pub component_item_code: String,
    pub component_item_description: Option<String>,
    pub quantity_required: String,
    pub quantity_issued: String,
    pub quantity_returned: String,
    pub quantity_scrapped: String,
    pub unit_of_measure: String,
    pub supply_type: String,
    pub supply_subinventory: Option<String>,
    pub wip_supply_type: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create Work Definition Request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateWorkDefinitionRequest {
    pub definition_number: Option<String>,
    pub description: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub production_type: Option<String>,
    pub planning_type: Option<String>,
    pub standard_lot_size: Option<String>,
    pub unit_of_measure: Option<String>,
    pub lead_time_days: Option<i32>,
    pub cost_type: Option<String>,
    pub standard_cost: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
}

/// Create Work Order Request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateWorkOrderRequest {
    pub work_definition_id: Option<Uuid>,
    pub description: Option<String>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity_ordered: String,
    pub unit_of_measure: Option<String>,
    pub scheduled_start_date: Option<chrono::NaiveDate>,
    pub scheduled_completion_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub priority: Option<String>,
    pub production_line: Option<String>,
    pub work_center_code: Option<String>,
    pub warehouse_code: Option<String>,
    pub cost_type: Option<String>,
    pub source_type: Option<String>,
    pub source_document_number: Option<String>,
    pub firm_planned: Option<bool>,
    pub company_id: Option<Uuid>,
    pub plant_code: Option<String>,
    pub created_by: Option<Uuid>,
}

/// Report Production Completion Request
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReportCompletionRequest {
    pub operation_sequence: Option<i32>,
    pub quantity_completed: String,
    pub quantity_scrapped: String,
    pub actual_run_hours: Option<String>,
    pub actual_labor_cost: Option<String>,
    pub actual_overhead_cost: Option<String>,
    pub completed_by: Option<Uuid>,
}

/// Issue Materials Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueMaterialRequest {
    pub material_id: Uuid,
    pub quantity_issued: String,
}

/// Manufacturing Dashboard Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManufacturingDashboard {
    pub total_work_orders: i32,
    pub open_work_orders: i32,
    pub in_progress_work_orders: i32,
    pub completed_work_orders: i32,
    pub cancelled_work_orders: i32,
    pub total_definitions: i32,
    pub active_definitions: i32,
    pub overdue_orders: i32,
    pub total_estimated_cost: String,
    pub total_actual_cost: String,
    pub cost_variance_pct: String,
    pub orders_by_status: serde_json::Value,
    pub orders_by_priority: serde_json::Value,
    pub completion_rate_pct: String,
    pub on_time_completion_pct: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Warehouse Management (Oracle Fusion Cloud Warehouse Management)
// ═══════════════════════════════════════════════════════════════════════════════

/// Warehouse definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Warehouse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub location_code: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse zone (e.g., receiving, bulk storage, picking, packing, staging)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseZone {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub warehouse_id: Uuid,
    pub code: String,
    pub name: String,
    pub zone_type: String, // receiving, storage, picking, packing, staging, shipping
    pub description: Option<String>,
    pub aisle_count: Option<i32>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Put-away rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PutAwayRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub warehouse_id: Uuid,
    pub rule_name: String,
    pub description: Option<String>,
    pub priority: i32,
    /// Item category filter (optional – null means all categories)
    pub item_category: Option<String>,
    /// Zone type to route to
    pub target_zone_type: String,
    /// Strategy: closest, zone_rotation, fixed_location
    pub strategy: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse task types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseTask {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub warehouse_id: Uuid,
    pub task_number: String,
    pub task_type: String, // pick, pack, put_away, load, receive
    pub status: String,   // pending, in_progress, completed, cancelled
    pub priority: String, // low, medium, high, urgent
    pub source_document: Option<String>,
    pub source_document_id: Option<Uuid>,
    pub source_line_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_description: Option<String>,
    pub from_zone_id: Option<Uuid>,
    pub to_zone_id: Option<Uuid>,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
    pub quantity: Option<String>,
    pub uom: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub wave_id: Option<Uuid>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Pick wave for grouping picking tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickWave {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub warehouse_id: Uuid,
    pub wave_number: String,
    pub status: String, // draft, released, in_progress, completed, cancelled
    pub priority: String,
    pub cut_off_date: Option<chrono::NaiveDate>,
    pub shipping_method: Option<String>,
    pub total_tasks: i32,
    pub completed_tasks: i32,
    pub released_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Warehouse dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseDashboard {
    pub total_warehouses: i64,
    pub active_warehouses: i64,
    pub total_zones: i64,
    pub total_pending_tasks: i64,
    pub total_in_progress_tasks: i64,
    pub total_completed_tasks_today: i64,
    pub total_active_waves: i64,
    pub tasks_by_type: serde_json::Value,
    pub tasks_by_priority: serde_json::Value,
    pub wave_completion_pct: String,
    pub recent_tasks: Vec<WarehouseTask>,
}

// ═══════════════════════════════════════════════════════════════════════
// Absence Management (Oracle Fusion Cloud HCM Absence Management)
// ═══════════════════════════════════════════════════════════════════════

/// Absence type definition (e.g., vacation, sick, parental)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceType {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub plan_type: String,
    pub requires_approval: bool,
    pub requires_documentation: bool,
    pub auto_approve_below_days: String,
    pub allow_negative_balance: bool,
    pub allow_half_day: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Absence plan with accrual rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsencePlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub absence_type_id: Uuid,
    pub accrual_frequency: String,
    pub accrual_rate: String,
    pub accrual_unit: String,
    pub carry_over_max: Option<String>,
    pub carry_over_expiry_months: Option<i32>,
    pub max_balance: Option<String>,
    pub probation_period_days: i32,
    pub prorate_first_year: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Employee absence balance for a given period
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceBalance {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub employee_id: Uuid,
    pub plan_id: Uuid,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub accrued: String,
    pub taken: String,
    pub adjusted: String,
    pub carried_over: String,
    pub remaining: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Individual absence entry (leave request/record)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub absence_type_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub entry_number: String,
    pub status: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub duration_days: String,
    pub duration_hours: Option<String>,
    pub is_half_day: bool,
    pub half_day_period: Option<String>,
    pub reason: Option<String>,
    pub comments: Option<String>,
    pub documentation_provided: bool,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub cancelled_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Absence entry history (audit trail)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceEntryHistory {
    pub id: Uuid,
    pub entry_id: Uuid,
    pub action: String,
    pub from_status: Option<String>,
    pub to_status: Option<String>,
    pub performed_by: Option<Uuid>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Absence management dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbsenceDashboard {
    pub total_types: i64,
    pub active_types: i64,
    pub total_plans: i64,
    pub active_plans: i64,
    pub pending_entries: i64,
    pub approved_entries_today: i64,
    pub entries_by_status: serde_json::Value,
    pub entries_by_type: serde_json::Value,
    pub recent_entries: Vec<AbsenceEntry>,
}

// ============================================================================
// Time and Labor Management (Oracle Fusion Cloud HCM Time and Labor)
// ============================================================================

/// Work schedule definition
/// Oracle Fusion: HCM > Time and Labor > Work Schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub schedule_type: String,
    pub standard_hours_per_day: String,
    pub standard_hours_per_week: String,
    pub work_days_per_week: i32,
    pub start_time: Option<chrono::NaiveTime>,
    pub end_time: Option<chrono::NaiveTime>,
    pub break_duration_minutes: i32,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Overtime rule definition
/// Oracle Fusion: HCM > Time and Labor > Overtime Rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OvertimeRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub threshold_type: String,
    pub daily_threshold_hours: String,
    pub weekly_threshold_hours: String,
    pub overtime_multiplier: String,
    pub double_time_threshold_hours: Option<String>,
    pub double_time_multiplier: String,
    pub include_holidays: bool,
    pub include_weekends: bool,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Time card (one per employee per period)
/// Oracle Fusion: HCM > Time and Labor > Time Cards
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeCard {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub card_number: String,
    pub status: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub total_regular_hours: String,
    pub total_overtime_hours: String,
    pub total_double_time_hours: String,
    pub total_hours: String,
    pub schedule_id: Option<Uuid>,
    pub overtime_rule_id: Option<Uuid>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub comments: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Time entry (individual time punch within a time card)
/// Oracle Fusion: HCM > Time and Labor > Time Entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub time_card_id: Uuid,
    pub entry_date: chrono::NaiveDate,
    pub entry_type: String,
    pub start_time: Option<chrono::NaiveTime>,
    pub end_time: Option<chrono::NaiveTime>,
    pub duration_hours: String,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub task_name: Option<String>,
    pub location: Option<String>,
    pub cost_center: Option<String>,
    pub labor_category: Option<String>,
    pub comments: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Time card history entry (audit trail)
/// Oracle Fusion: HCM > Time and Labor > Time Card History
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeCardHistory {
    pub id: Uuid,
    pub time_card_id: Uuid,
    pub action: String,
    pub from_status: Option<String>,
    pub to_status: Option<String>,
    pub performed_by: Option<Uuid>,
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Labor distribution (cost allocation for time entries)
/// Oracle Fusion: HCM > Time and Labor > Labor Distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaborDistribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub time_entry_id: Uuid,
    pub distribution_percent: String,
    pub cost_center: Option<String>,
    pub project_id: Option<Uuid>,
    pub project_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub gl_account_code: Option<String>,
    pub allocated_hours: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Time and Labor dashboard summary
/// Oracle Fusion: HCM > Time and Labor > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeAndLaborDashboard {
    pub total_schedules: i64,
    pub active_schedules: i64,
    pub total_overtime_rules: i64,
    pub total_time_cards: i64,
    pub pending_approval_count: i64,
    pub submitted_today_count: i64,
    pub cards_by_status: serde_json::Value,
    pub hours_by_type: serde_json::Value,
    pub recent_time_cards: Vec<TimeCard>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Approval Authority Limits
// Oracle Fusion: BPM > Approval Configuration > Document Approval Limits
// ═══════════════════════════════════════════════════════════════════════════════

/// Approval authority limit – defines the maximum monetary amount a user
/// or role is authorised to approve for a given document type.
///
/// Oracle Fusion Cloud calls these "Document Approval Limits" or
/// "Signing Limits". They restrict who can approve what, up to how much,
/// for which business unit / cost center.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalAuthorityLimit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub limit_code: String,
    pub name: String,
    pub description: Option<String>,
    /// "user" or "role"
    pub owner_type: String,
    /// User ID when owner_type = "user"
    pub user_id: Option<Uuid>,
    /// Role name when owner_type = "role"
    pub role_name: Option<String>,
    /// Document type this limit applies to
    pub document_type: String,
    /// Maximum amount the owner can approve in a single transaction
    pub approval_limit_amount: String,
    /// Currency code (e.g. "USD")
    pub currency_code: String,
    /// Optional business unit scope
    pub business_unit_id: Option<Uuid>,
    /// Optional cost center scope
    pub cost_center: Option<String>,
    /// "active" or "inactive"
    pub status: String,
    /// Effective date range
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Audit trail entry for authority-limit checks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityCheckAudit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub limit_id: Option<Uuid>,
    /// The user attempting the approval
    pub checked_user_id: Uuid,
    /// The role being checked (if role-based)
    pub checked_role: Option<String>,
    /// Document type
    pub document_type: String,
    /// The document being approved
    pub document_id: Option<Uuid>,
    /// The amount being approved
    pub requested_amount: String,
    /// The limit that was found (or 0 if none)
    pub applicable_limit: String,
    /// "approved", "denied"
    pub result: String,
    /// Reason for the result
    pub reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Request to create an approval authority limit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApprovalAuthorityLimitRequest {
    pub limit_code: String,
    pub name: String,
    pub description: Option<String>,
    /// "user" or "role"
    pub owner_type: String,
    pub user_id: Option<String>,
    pub role_name: Option<String>,
    pub document_type: String,
    pub approval_limit_amount: String,
    pub currency_code: String,
    pub business_unit_id: Option<String>,
    pub cost_center: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Dashboard summary for approval authority limits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalAuthorityDashboard {
    pub total_limits: i64,
    pub active_limits: i64,
    pub limits_by_document_type: serde_json::Value,
    pub limits_by_owner_type: serde_json::Value,
    pub recent_checks: Vec<AuthorityCheckAudit>,
    pub total_checks: i64,
    pub approved_checks: i64,
    pub denied_checks: i64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Data Archiving and Retention Management
// Oracle Fusion: Information Lifecycle Management (ILM)
// ═══════════════════════════════════════════════════════════════════════════════

/// Retention policy – defines how long data of a given entity type
/// must be retained before it can be archived or purged.
///
/// Oracle Fusion Cloud: Tools > Information Lifecycle Management > Retention Policies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub policy_code: String,
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
    pub retention_days: i32,
    /// "archive", "purge", "archive_then_purge"
    pub action_type: String,
    pub purge_after_days: Option<i32>,
    pub condition_expression: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create/update a retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRetentionPolicyRequest {
    pub policy_code: String,
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
    #[serde(default = "default_365")]
    pub retention_days: i32,
    #[serde(default = "default_action_type")]
    pub action_type: String,
    pub purge_after_days: Option<i32>,
    pub condition_expression: Option<String>,
}

fn default_365() -> i32 { 365 }
fn default_action_type() -> String { "archive_then_purge".to_string() }

/// Legal hold – prevents archival or purging of specific records.
///
/// Oracle Fusion Cloud: Information Lifecycle Management > Legal Holds
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalHold {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub hold_number: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub reason: Option<String>,
    pub case_reference: Option<String>,
    pub authorized_by: Option<Uuid>,
    pub released_at: Option<DateTime<Utc>>,
    pub released_by: Option<Uuid>,
    pub release_reason: Option<String>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a legal hold
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLegalHoldRequest {
    pub hold_number: String,
    pub name: String,
    pub description: Option<String>,
    pub reason: Option<String>,
    pub case_reference: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
}

/// Legal hold item – a specific record under a legal hold
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalHoldItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub legal_hold_id: Uuid,
    pub entity_type: String,
    pub record_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Archived record – tracks a record that has been archived
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivedRecord {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entity_type: String,
    pub original_record_id: Uuid,
    pub original_data: serde_json::Value,
    pub retention_policy_id: Option<Uuid>,
    pub archive_batch_id: Option<Uuid>,
    pub status: String,
    pub original_created_at: Option<DateTime<Utc>>,
    pub original_updated_at: Option<DateTime<Utc>>,
    pub archived_at: DateTime<Utc>,
    pub archived_by: Option<Uuid>,
    pub restored_at: Option<DateTime<Utc>>,
    pub restored_by: Option<Uuid>,
    pub purged_at: Option<DateTime<Utc>>,
    pub purged_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Archive batch – groups records archived together
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveBatch {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub batch_number: String,
    pub retention_policy_id: Option<Uuid>,
    pub entity_type: String,
    pub status: String,
    pub total_records: i32,
    pub archived_records: i32,
    pub failed_records: i32,
    pub criteria: serde_json::Value,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Archive audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveAudit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub operation: String,
    pub entity_type: String,
    pub record_id: Option<Uuid>,
    pub batch_id: Option<Uuid>,
    pub legal_hold_id: Option<Uuid>,
    pub retention_policy_id: Option<Uuid>,
    pub result: String,
    pub details: Option<String>,
    pub performed_by: Option<Uuid>,
    pub performed_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Data archiving dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataArchivingDashboard {
    pub total_policies: i64,
    pub active_policies: i64,
    pub total_legal_holds: i64,
    pub active_legal_holds: i64,
    pub total_archived_records: i64,
    pub total_purged_records: i64,
    pub total_restored_records: i64,
    pub policies_by_entity_type: serde_json::Value,
    pub recent_audit_entries: Vec<ArchiveAudit>,
}

// ============================================================================
// Payroll Management (Oracle Fusion Global Payroll)
// ============================================================================

/// Payroll definition — represents a pay group.
/// Oracle Fusion: Payroll > Payroll Definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// How often this payroll runs: "weekly", "biweekly", "semimonthly", "monthly"
    pub pay_frequency: String,
    /// Default currency code for payroll runs
    pub currency_code: String,
    /// GL account code for salary expense posting
    pub salary_expense_account: Option<String>,
    /// GL account code for employer liabilities
    pub liability_account: Option<String>,
    /// GL account code for employer tax expense
    pub employer_tax_account: Option<String>,
    /// GL account code for bank / cash disbursement
    pub payment_account: Option<String>,
    /// Whether this payroll definition is active
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Payroll element — an earning or deduction component.
/// Oracle Fusion: Payroll > Element Definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollElement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    /// "earning" or "deduction"
    pub element_type: String,
    /// "salary", "hourly", "overtime", "bonus", "commission", "benefit", "tax", "retirement", "garnishment", "other"
    pub category: String,
    /// How the value is determined: "flat", "percentage", "hourly_rate", "formula"
    pub calculation_method: String,
    /// Default rate / value depending on calculation_method
    pub default_value: Option<String>,
    /// Whether this element is recurring every pay period
    pub is_recurring: bool,
    /// Whether employer also contributes (e.g. employer match on 401k)
    pub has_employer_contribution: bool,
    /// Percentage for employer contribution (if applicable)
    pub employer_contribution_rate: Option<String>,
    /// GL account override for this element
    pub gl_account_code: Option<String>,
    /// Whether this element is pretax (deducted before tax calc)
    pub is_pretax: bool,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Employee element assignment — links an element to an employee with a value.
/// Oracle Fusion: Payroll > Element Entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollElementEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub employee_id: Uuid,
    pub element_id: Uuid,
    pub element_code: String,
    pub element_name: String,
    pub element_type: String,
    pub entry_value: String,
    /// How many periods this entry spans (None = indefinite)
    pub remaining_periods: Option<i32>,
    pub is_active: bool,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Payroll run — a single execution of payroll for a period.
/// Oracle Fusion: Payroll > Payroll Runs / Quick Pay
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payroll_id: Uuid,
    pub run_number: String,
    /// "open", "calculated", "confirmed", "paid", "reversed"
    pub status: String,
    /// Period being paid
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    /// When payroll should be deposited
    pub pay_date: chrono::NaiveDate,
    pub total_gross: String,
    pub total_deductions: String,
    pub total_net: String,
    pub total_employer_cost: String,
    pub employee_count: i32,
    pub confirmed_by: Option<Uuid>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub paid_by: Option<Uuid>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Pay slip — per-employee payroll result within a run.
/// Oracle Fusion: Payroll > Pay Slips / Payment History
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaySlip {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub payroll_run_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub gross_earnings: String,
    pub total_deductions: String,
    pub net_pay: String,
    pub employer_cost: String,
    pub currency_code: String,
    pub payment_method: Option<String>,
    pub bank_account_last4: Option<String>,
    pub lines: Vec<PaySlipLine>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Individual line on a pay slip (one earning or deduction).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaySlipLine {
    pub id: Uuid,
    pub pay_slip_id: Uuid,
    pub element_code: String,
    pub element_name: String,
    pub element_type: String,
    pub category: String,
    /// Number of hours (for hourly earnings), units, or quantity
    pub hours_or_units: Option<String>,
    /// Rate per unit / hour
    pub rate: Option<String>,
    /// Computed amount for this line
    pub amount: String,
    /// Whether this is pretax
    pub is_pretax: bool,
    /// Whether this is an employer-paid portion
    pub is_employer: bool,
    pub gl_account_code: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Payroll summary for dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PayrollDashboard {
    pub total_gross: String,
    pub total_deductions: String,
    pub total_net: String,
    pub total_employer_cost: String,
    pub employee_count: i32,
    pub payroll_runs_this_period: i32,
    pub recent_runs: Vec<PayrollRun>,
    pub top_earnings_by_category: serde_json::Value,
    pub top_deductions_by_category: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Compensation Management (Oracle Fusion Cloud HCM Compensation Workbench)
// ═══════════════════════════════════════════════════════════════════════════════

/// Compensation plan component type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationComponent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Uuid,
    pub component_name: String,
    pub component_type: String,
    pub description: Option<String>,
    pub is_recurring: bool,
    pub frequency: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation plan definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_code: String,
    pub plan_name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub status: String,
    pub effective_start_date: Option<chrono::NaiveDate>,
    pub effective_end_date: Option<chrono::NaiveDate>,
    pub eligibility_criteria: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation cycle (annual review cycle)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationCycle {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cycle_name: String,
    pub description: Option<String>,
    pub cycle_type: String,
    pub status: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub allocation_start_date: Option<chrono::NaiveDate>,
    pub allocation_end_date: Option<chrono::NaiveDate>,
    pub review_start_date: Option<chrono::NaiveDate>,
    pub review_end_date: Option<chrono::NaiveDate>,
    pub total_budget: String,
    pub total_allocated: String,
    pub total_approved: String,
    pub total_employees: i32,
    pub currency_code: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation budget pool for manager allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationBudgetPool {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cycle_id: Uuid,
    pub pool_name: String,
    pub pool_type: String,
    pub manager_id: Option<Uuid>,
    pub manager_name: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub total_budget: String,
    pub allocated_amount: String,
    pub approved_amount: String,
    pub remaining_budget: String,
    pub currency_code: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation worksheet line (per-employee allocation)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationWorksheetLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub worksheet_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub job_title: Option<String>,
    pub department_name: Option<String>,
    pub current_base_salary: String,
    pub proposed_base_salary: String,
    pub salary_change_amount: String,
    pub salary_change_percent: String,
    pub merit_amount: String,
    pub bonus_amount: String,
    pub equity_amount: String,
    pub total_compensation: String,
    pub performance_rating: Option<String>,
    pub compa_ratio: String,
    pub status: String,
    pub manager_comments: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation worksheet (manager's worksheet)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationWorksheet {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cycle_id: Uuid,
    pub pool_id: Option<Uuid>,
    pub manager_id: Uuid,
    pub manager_name: Option<String>,
    pub status: String,
    pub total_employees: i32,
    pub total_current_salary: String,
    pub total_proposed_salary: String,
    pub total_merit: String,
    pub total_bonus: String,
    pub total_equity: String,
    pub total_compensation_change: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation statement (employee view)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationStatement {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub cycle_id: Uuid,
    pub employee_id: Uuid,
    pub employee_name: Option<String>,
    pub statement_date: chrono::NaiveDate,
    pub base_salary: String,
    pub merit_increase: String,
    pub bonus: String,
    pub equity: String,
    pub benefits_value: String,
    pub total_compensation: String,
    pub total_direct_compensation: String,
    pub total_indirect_compensation: String,
    pub change_from_previous: String,
    pub change_percent: String,
    pub currency_code: String,
    pub components: serde_json::Value,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Compensation dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompensationDashboard {
    pub active_plans: i32,
    pub active_cycles: i32,
    pub total_budget: String,
    pub total_allocated: String,
    pub total_approved: String,
    pub total_employees_in_cycle: i32,
    pub pending_worksheets: i32,
    pub completed_worksheets: i32,
    pub average_salary_increase_percent: String,
    pub budget_utilization_percent: String,
}

// ============================================================================
// Service Request Management (Oracle Fusion CX Service)
// ============================================================================

/// Service request category
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub default_priority: Option<String>,
    pub default_sla_hours: Option<i32>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Service request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_number: String,
    pub title: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub category_name: Option<String>,
    pub priority: String,
    pub status: String,
    pub request_type: String,
    pub channel: String,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub assigned_group: Option<String>,
    pub product_id: Option<Uuid>,
    pub product_name: Option<String>,
    pub serial_number: Option<String>,
    pub resolution: Option<String>,
    pub resolution_code: Option<String>,
    pub sla_due_date: Option<chrono::NaiveDate>,
    pub sla_breached: bool,
    pub parent_request_id: Option<Uuid>,
    pub related_object_type: Option<String>,
    pub related_object_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Service request communication/update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequestUpdate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_id: Uuid,
    pub update_type: String,
    pub author_id: Option<Uuid>,
    pub author_name: Option<String>,
    pub subject: Option<String>,
    pub body: String,
    pub is_internal: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Service request assignment history
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequestAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub request_id: Uuid,
    pub assigned_to: Option<Uuid>,
    pub assigned_to_name: Option<String>,
    pub assigned_group: Option<String>,
    pub assigned_by: Option<Uuid>,
    pub assigned_by_name: Option<String>,
    pub assignment_type: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Service request dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRequestDashboard {
    pub total_open: i32,
    pub total_resolved: i32,
    pub total_closed: i32,
    pub total_unassigned: i32,
    pub sla_breached_count: i32,
    pub by_priority: serde_json::Value,
    pub by_status: serde_json::Value,
    pub by_category: serde_json::Value,
    pub by_channel: serde_json::Value,
    pub average_resolution_hours: String,
}

// ============================================================================
// Lead and Opportunity Management (Oracle Fusion CX Sales)
// ============================================================================

/// Lead source (e.g. website, referral, trade show)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadSource {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lead rating / scoring model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeadRatingModel {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub scoring_criteria: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sales lead
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesLead {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub lead_number: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub lead_source_id: Option<Uuid>,
    pub lead_source_name: Option<String>,
    pub lead_rating_model_id: Option<Uuid>,
    pub lead_score: String,
    pub lead_rating: String,
    pub estimated_value: String,
    pub currency_code: String,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub converted_opportunity_id: Option<Uuid>,
    pub converted_customer_id: Option<Uuid>,
    pub converted_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub address: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Opportunity pipeline stage
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpportunityStage {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub probability: String,
    pub display_order: i32,
    pub is_won: bool,
    pub is_lost: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sales opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesOpportunity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub opportunity_number: String,
    pub name: String,
    pub description: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub lead_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub stage_name: Option<String>,
    pub amount: String,
    pub currency_code: String,
    pub probability: String,
    pub weighted_amount: String,
    pub expected_close_date: Option<chrono::NaiveDate>,
    pub actual_close_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub competitor: Option<String>,
    pub lost_reason: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Opportunity line item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpportunityLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub opportunity_id: Uuid,
    pub line_number: i32,
    pub product_name: String,
    pub product_code: Option<String>,
    pub description: Option<String>,
    pub quantity: String,
    pub unit_price: String,
    pub line_amount: String,
    pub discount_percent: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Sales activity (call, meeting, task)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesActivity {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub subject: String,
    pub description: Option<String>,
    pub activity_type: String,
    pub status: String,
    pub priority: String,
    pub lead_id: Option<Uuid>,
    pub opportunity_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub outcome: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Opportunity stage history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpportunityStageHistory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub opportunity_id: Uuid,
    pub from_stage: Option<String>,
    pub to_stage: String,
    pub changed_by: Option<Uuid>,
    pub changed_by_name: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub notes: Option<String>,
}

/// Lead and Opportunity dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SalesPipelineDashboard {
    pub total_leads: i32,
    pub new_leads: i32,
    pub qualified_leads: i32,
    pub converted_leads: i32,
    pub total_opportunities: i32,
    pub open_opportunities: i32,
    pub won_opportunities: i32,
    pub lost_opportunities: i32,
    pub total_pipeline_value: String,
    pub weighted_pipeline_value: String,
    pub total_won_value: String,
    pub average_deal_size: String,
    pub win_rate: String,
    pub by_stage: serde_json::Value,
    pub by_owner: serde_json::Value,
}

// ============================================================================
// Demand Planning / Demand Management (Oracle Fusion SCM > Demand Management)
// ============================================================================

/// Forecast method definition
/// Oracle Fusion: Demand Management > Forecast Methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandForecastMethod {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub method_type: String,
    pub parameters: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Demand schedule (forecast header)
/// Oracle Fusion: Demand Management > Demand Schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandSchedule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_number: String,
    pub name: String,
    pub description: Option<String>,
    pub method_id: Option<Uuid>,
    pub method_name: Option<String>,
    pub schedule_type: String,
    pub status: String,
    pub start_date: chrono::NaiveDate,
    pub end_date: chrono::NaiveDate,
    pub currency_code: String,
    pub total_forecast_quantity: String,
    pub total_forecast_value: String,
    pub confidence_level: String,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Demand schedule line (forecast item per period)
/// Oracle Fusion: Demand Management > Schedule Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandScheduleLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_id: Uuid,
    pub line_number: i32,
    pub item_code: String,
    pub item_name: Option<String>,
    pub item_category: Option<String>,
    pub warehouse_code: Option<String>,
    pub region: Option<String>,
    pub customer_group: Option<String>,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub forecast_quantity: String,
    pub forecast_value: String,
    pub unit_price: String,
    pub consumed_quantity: String,
    pub remaining_quantity: String,
    pub confidence_pct: String,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Historical demand data (actuals)
/// Oracle Fusion: Demand Management > Demand History
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandHistory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_code: String,
    pub item_name: Option<String>,
    pub warehouse_code: Option<String>,
    pub region: Option<String>,
    pub customer_group: Option<String>,
    pub actual_date: chrono::NaiveDate,
    pub actual_quantity: String,
    pub actual_value: String,
    pub source_type: String,
    pub source_id: Option<Uuid>,
    pub source_line_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Forecast consumption entry
/// Oracle Fusion: Demand Management > Forecast Consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandConsumption {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_line_id: Uuid,
    pub history_id: Option<Uuid>,
    pub consumed_quantity: String,
    pub consumed_date: chrono::NaiveDate,
    pub source_type: String,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Forecast accuracy measurement
/// Oracle Fusion: Demand Management > Accuracy Analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandAccuracy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub schedule_id: Uuid,
    pub schedule_line_id: Option<Uuid>,
    pub item_code: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub forecast_quantity: String,
    pub actual_quantity: String,
    pub absolute_error: String,
    pub absolute_pct_error: String,
    pub bias: String,
    pub measurement_date: chrono::NaiveDate,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Demand Planning Dashboard
/// Oracle Fusion: Demand Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemandPlanningDashboard {
    pub total_schedules: i32,
    pub active_schedules: i32,
    pub total_forecast_items: i32,
    pub total_forecast_quantity: String,
    pub total_forecast_value: String,
    pub avg_accuracy_pct: String,
    pub schedules_by_status: serde_json::Value,
    pub top_forecast_items: serde_json::Value,
    pub accuracy_by_method: serde_json::Value,
}

// ============================================================================
// Shipping Execution (Oracle Fusion SCM > Shipping Execution)
// ============================================================================

/// Shipping Carrier
/// Oracle Fusion: SCM > Shipping > Carriers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShippingCarrier {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub carrier_type: String,
    pub tracking_url_template: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Shipping Method
/// Oracle Fusion: SCM > Shipping > Shipping Methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShippingMethod {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub carrier_id: Option<Uuid>,
    pub transit_time_days: i32,
    pub is_express: bool,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Shipment (shipping header)
/// Oracle Fusion: SCM > Shipping > Shipments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shipment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_number: String,
    pub description: Option<String>,
    pub status: String,
    pub carrier_id: Option<Uuid>,
    pub carrier_name: Option<String>,
    pub shipping_method_id: Option<Uuid>,
    pub shipping_method_name: Option<String>,
    pub order_id: Option<Uuid>,
    pub order_number: Option<String>,
    pub customer_id: Option<Uuid>,
    pub customer_name: Option<String>,
    pub ship_from_warehouse: Option<String>,
    pub ship_to_name: Option<String>,
    pub ship_to_address: Option<String>,
    pub ship_to_city: Option<String>,
    pub ship_to_state: Option<String>,
    pub ship_to_postal_code: Option<String>,
    pub ship_to_country: Option<String>,
    pub tracking_number: Option<String>,
    pub total_weight: String,
    pub weight_unit: String,
    pub total_volume: String,
    pub volume_unit: String,
    pub total_packages: i32,
    pub shipped_date: Option<DateTime<Utc>>,
    pub estimated_delivery: Option<chrono::NaiveDate>,
    pub actual_delivery: Option<DateTime<Utc>>,
    pub confirmed_by: Option<Uuid>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub shipped_by: Option<Uuid>,
    pub delivered_by: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Shipment Line
/// Oracle Fusion: SCM > Shipping > Shipment Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_id: Uuid,
    pub line_number: i32,
    pub order_line_id: Option<Uuid>,
    pub item_code: String,
    pub item_name: Option<String>,
    pub item_description: Option<String>,
    pub requested_quantity: String,
    pub shipped_quantity: String,
    pub backordered_quantity: String,
    pub unit_of_measure: String,
    pub weight: String,
    pub weight_unit: String,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub is_fragile: bool,
    pub is_hazardous: bool,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Packing Slip
/// Oracle Fusion: SCM > Shipping > Packing Slips
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackingSlip {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub shipment_id: Uuid,
    pub packing_slip_number: String,
    pub package_number: i32,
    pub package_type: String,
    pub weight: String,
    pub weight_unit: String,
    pub dimensions_length: String,
    pub dimensions_width: String,
    pub dimensions_height: String,
    pub dimensions_unit: String,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Packing Slip Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackingSlipLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub packing_slip_id: Uuid,
    pub shipment_line_id: Uuid,
    pub line_number: i32,
    pub item_code: String,
    pub item_name: Option<String>,
    pub packed_quantity: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Shipping Execution Dashboard
/// Oracle Fusion: SCM > Shipping > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShippingDashboard {
    pub total_shipments: i32,
    pub pending_shipments: i32,
    pub shipped_this_month: i32,
    pub delivered_this_month: i32,
    pub total_carriers: i32,
    pub shipments_by_status: serde_json::Value,
    pub recent_shipments: serde_json::Value,
    pub top_carriers: serde_json::Value,
}

// ============================================================================
// Recruiting Management (Oracle Fusion HCM > Recruiting)
// ============================================================================

/// Job Requisition
/// Oracle Fusion: HCM > Recruiting > Job Requisitions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRequisition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub requisition_number: String,
    pub title: String,
    pub description: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employment_type: String,
    pub position_type: String,
    pub vacancies: i32,
    pub priority: String,
    pub salary_min: Option<String>,
    pub salary_max: Option<String>,
    pub currency: String,
    pub required_skills: serde_json::Value,
    pub qualifications: Option<String>,
    pub experience_years_min: Option<i32>,
    pub experience_years_max: Option<i32>,
    pub education_level: Option<String>,
    pub hiring_manager_id: Option<Uuid>,
    pub recruiter_id: Option<Uuid>,
    pub target_start_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub posted_date: Option<DateTime<Utc>>,
    pub closed_date: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Candidate
/// Oracle Fusion: HCM > Recruiting > Candidates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub candidate_number: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub linkedin_url: Option<String>,
    pub source: Option<String>,
    pub source_detail: Option<String>,
    pub resume_url: Option<String>,
    pub cover_letter_url: Option<String>,
    pub current_employer: Option<String>,
    pub current_title: Option<String>,
    pub years_of_experience: Option<i32>,
    pub education_level: Option<String>,
    pub skills: serde_json::Value,
    pub notes: Option<String>,
    pub status: String,
    pub tags: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Job Application
/// Oracle Fusion: HCM > Recruiting > Job Applications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobApplication {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub application_number: Option<String>,
    pub requisition_id: Uuid,
    pub candidate_id: Uuid,
    pub status: String,
    pub match_score: String,
    pub screening_notes: Option<String>,
    pub rejection_reason: Option<String>,
    pub applied_at: DateTime<Utc>,
    pub last_status_change: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Interview
/// Oracle Fusion: HCM > Recruiting > Interviews
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interview {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub interview_type: String,
    pub round: i32,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub duration_minutes: i32,
    pub location: Option<String>,
    pub meeting_link: Option<String>,
    pub interviewer_ids: serde_json::Value,
    pub interviewer_names: serde_json::Value,
    pub status: String,
    pub feedback: Option<String>,
    pub rating: Option<i32>,
    pub recommendation: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Job Offer
/// Oracle Fusion: HCM > Recruiting > Job Offers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobOffer {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub application_id: Uuid,
    pub offer_number: Option<String>,
    pub job_title: String,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employment_type: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub salary_offered: Option<String>,
    pub salary_currency: String,
    pub salary_frequency: String,
    pub signing_bonus: Option<String>,
    pub benefits_summary: Option<String>,
    pub terms_and_conditions: Option<String>,
    pub status: String,
    pub offer_date: Option<DateTime<Utc>>,
    pub response_deadline: Option<DateTime<Utc>>,
    pub responded_at: Option<DateTime<Utc>>,
    pub response_notes: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Recruiting Dashboard
/// Oracle Fusion: HCM > Recruiting > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitingDashboard {
    pub total_requisitions: i32,
    pub open_requisitions: i32,
    pub total_candidates: i32,
    pub total_applications: i32,
    pub applications_this_month: i32,
    pub interviews_this_month: i32,
    pub offers_pending: i32,
    pub hires_this_month: i32,
    pub requisitions_by_status: serde_json::Value,
    pub applications_by_status: serde_json::Value,
    pub top_departments: serde_json::Value,
    pub recent_applications: serde_json::Value,
}

// ============================================================================
// Marketing Campaign Management (Oracle Fusion CX Marketing)
// ============================================================================

/// Campaign Type
/// Oracle Fusion: CX Marketing > Campaign Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignType {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub channel: String,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Marketing Campaign
/// Oracle Fusion: CX Marketing > Campaigns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketingCampaign {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub campaign_number: String,
    pub name: String,
    pub description: Option<String>,
    pub campaign_type_id: Option<Uuid>,
    pub campaign_type_name: Option<String>,
    pub status: String,
    pub channel: String,
    pub budget: String,
    pub actual_cost: String,
    pub currency_code: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub expected_responses: i32,
    pub expected_revenue: String,
    pub actual_responses: i32,
    pub actual_revenue: String,
    pub converted_leads: i32,
    pub converted_opportunities: i32,
    pub converted_won: i32,
    pub parent_campaign_id: Option<Uuid>,
    pub parent_campaign_name: Option<String>,
    pub tags: serde_json::Value,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub activated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Campaign Member
/// Oracle Fusion: CX Marketing > Campaign Members
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub campaign_id: Uuid,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub lead_id: Option<Uuid>,
    pub lead_number: Option<String>,
    pub status: String,
    pub response: Option<String>,
    pub responded_at: Option<DateTime<Utc>>,
    pub converted_contact_id: Option<Uuid>,
    pub converted_lead_id: Option<Uuid>,
    pub converted_opportunity_id: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Campaign Response
/// Oracle Fusion: CX Marketing > Campaign Responses
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub campaign_id: Uuid,
    pub member_id: Option<Uuid>,
    pub response_type: String,
    pub contact_id: Option<Uuid>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub lead_id: Option<Uuid>,
    pub description: Option<String>,
    pub value: String,
    pub currency_code: String,
    pub source_url: Option<String>,
    pub responded_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Marketing Dashboard
/// Oracle Fusion: CX Marketing > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketingDashboard {
    pub total_campaigns: i32,
    pub active_campaigns: i32,
    pub completed_campaigns: i32,
    pub total_budget: String,
    pub total_actual_cost: String,
    pub total_expected_revenue: String,
    pub total_actual_revenue: String,
    pub total_responses: i32,
    pub total_converted_leads: i32,
    pub overall_roi: String,
    pub campaigns_by_status: serde_json::Value,
    pub campaigns_by_channel: serde_json::Value,
    pub top_campaigns: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Receiving Management (Oracle Fusion SCM > Receiving)
// ═══════════════════════════════════════════════════════════════════════════════

/// Receiving Location
/// Oracle Fusion: SCM > Receiving > Locations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivingLocation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub location_type: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receipt Header
/// Oracle Fusion: SCM > Receiving > Receipts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptHeader {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_number: String,
    pub receipt_type: String,
    pub receipt_source: String,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub supplier_number: Option<String>,
    pub purchase_order_id: Option<Uuid>,
    pub purchase_order_number: Option<String>,
    pub receiving_location_id: Option<Uuid>,
    pub receiving_location_code: Option<String>,
    pub receiving_date: Option<chrono::NaiveDate>,
    pub packing_slip_number: Option<String>,
    pub bill_of_lading: Option<String>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub waybill_number: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub total_received_qty: String,
    pub total_inspected_qty: String,
    pub total_accepted_qty: String,
    pub total_rejected_qty: String,
    pub total_delivered_qty: String,
    pub received_by: Option<Uuid>,
    pub received_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receipt Line
/// Oracle Fusion: SCM > Receiving > Receipt Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_id: Uuid,
    pub line_number: i32,
    pub purchase_order_line_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub ordered_qty: String,
    pub ordered_uom: Option<String>,
    pub received_qty: String,
    pub received_uom: Option<String>,
    pub accepted_qty: String,
    pub rejected_qty: String,
    pub inspection_status: String,
    pub delivery_status: String,
    pub lot_number: Option<String>,
    pub serial_numbers: serde_json::Value,
    pub expiration_date: Option<chrono::NaiveDate>,
    pub manufacture_date: Option<chrono::NaiveDate>,
    pub unit_price: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receipt Inspection
/// Oracle Fusion: SCM > Receiving > Inspections
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptInspection {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_id: Uuid,
    pub receipt_line_id: Uuid,
    pub inspection_number: String,
    pub inspection_template: Option<String>,
    pub inspector_id: Option<Uuid>,
    pub inspector_name: Option<String>,
    pub inspection_date: Option<chrono::NaiveDate>,
    pub sample_size: Option<String>,
    pub quantity_inspected: String,
    pub quantity_accepted: String,
    pub quantity_rejected: String,
    pub disposition: String,
    pub rejection_reason: Option<String>,
    pub quality_score: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inspection Detail
/// Oracle Fusion: SCM > Receiving > Inspection Details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectionDetail {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub inspection_id: Uuid,
    pub check_number: i32,
    pub check_name: String,
    pub check_type: String,
    pub specification: Option<String>,
    pub result: String,
    pub measured_value: Option<String>,
    pub expected_value: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Receipt Delivery
/// Oracle Fusion: SCM > Receiving > Deliveries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptDelivery {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub receipt_id: Uuid,
    pub receipt_line_id: Uuid,
    pub delivery_number: String,
    pub subinventory: Option<String>,
    pub locator: Option<String>,
    pub quantity_delivered: String,
    pub uom: Option<String>,
    pub lot_number: Option<String>,
    pub serial_number: Option<String>,
    pub delivered_by: Option<Uuid>,
    pub delivered_by_name: Option<String>,
    pub delivery_date: Option<DateTime<Utc>>,
    pub destination_type: String,
    pub account_code: Option<String>,
    pub notes: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receipt Return (Return to Supplier)
/// Oracle Fusion: SCM > Receiving > Returns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptReturn {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub return_number: String,
    pub receipt_id: Option<Uuid>,
    pub receipt_line_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub return_type: String,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub quantity_returned: String,
    pub uom: Option<String>,
    pub unit_price: Option<String>,
    pub currency: Option<String>,
    pub return_reason: Option<String>,
    pub return_date: Option<chrono::NaiveDate>,
    pub carrier: Option<String>,
    pub tracking_number: Option<String>,
    pub credit_expected: bool,
    pub credit_memo_number: Option<String>,
    pub status: String,
    pub shipped_at: Option<DateTime<Utc>>,
    pub credited_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Receiving Dashboard
/// Oracle Fusion: SCM > Receiving > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivingDashboard {
    pub total_receipts: i32,
    pub pending_receipts: i32,
    pub received_today: i32,
    pub pending_inspections: i32,
    pub pending_deliveries: i32,
    pub total_returns: i32,
    pub receipts_by_status: serde_json::Value,
    pub top_suppliers: serde_json::Value,
    pub recent_receipts: serde_json::Value,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Supplier Scorecard Management (Oracle Fusion Supplier Portal > Supplier Performance)
// ═══════════════════════════════════════════════════════════════════════════════

/// Scorecard Template
/// Oracle Fusion: Supplier Portal > Performance > Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScorecardTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub evaluation_period: String,
    pub is_active: bool,
    pub total_weight: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Scorecard Category (KPI category within a template)
/// Oracle Fusion: Supplier Portal > Performance > KPI Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScorecardCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub weight: String,
    pub sort_order: i32,
    pub scoring_model: String,
    pub target_score: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Scorecard
/// Oracle Fusion: Supplier Portal > Performance > Scorecards
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierScorecard {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Uuid,
    pub scorecard_number: String,
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    pub supplier_number: Option<String>,
    pub evaluation_period_start: chrono::NaiveDate,
    pub evaluation_period_end: chrono::NaiveDate,
    pub status: String,
    pub overall_score: String,
    pub overall_grade: Option<String>,
    pub reviewer_id: Option<Uuid>,
    pub reviewer_name: Option<String>,
    pub review_date: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Scorecard Line (individual KPI score)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScorecardLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub scorecard_id: Uuid,
    pub category_id: Uuid,
    pub line_number: i32,
    pub kpi_name: String,
    pub kpi_description: Option<String>,
    pub weight: String,
    pub target_value: Option<String>,
    pub actual_value: Option<String>,
    pub score: String,
    pub weighted_score: String,
    pub evidence: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Performance Review
/// Oracle Fusion: Supplier Portal > Performance > Reviews
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierPerformanceReview {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub review_number: String,
    pub supplier_id: Uuid,
    pub supplier_name: Option<String>,
    pub scorecard_id: Option<Uuid>,
    pub review_type: String,
    pub review_period: Option<String>,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub previous_score: Option<String>,
    pub current_score: Option<String>,
    pub score_change: Option<String>,
    pub rating: Option<String>,
    pub strengths: Option<String>,
    pub improvement_areas: Option<String>,
    pub action_items: Option<String>,
    pub follow_up_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub reviewer_id: Option<Uuid>,
    pub reviewer_name: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Review Action Item
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewActionItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub review_id: Uuid,
    pub action_number: i32,
    pub description: String,
    pub assignee_id: Option<Uuid>,
    pub assignee_name: Option<String>,
    pub priority: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub completed_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Supplier Scorecard Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierScorecardDashboard {
    pub total_templates: i32,
    pub total_scorecards: i32,
    pub pending_reviews: i32,
    pub average_score: String,
    pub scorecards_by_status: serde_json::Value,
    pub scorecards_by_grade: serde_json::Value,
    pub top_performers: serde_json::Value,
    pub bottom_performers: serde_json::Value,
    pub recent_reviews: serde_json::Value,
}

// ============================================================================
// KPI & Embedded Analytics (Oracle Fusion OTBI-inspired)
// ============================================================================

/// KPI definition
/// Oracle Fusion: Analytics > KPI Library
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KpiDefinition {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub unit_of_measure: String,
    pub direction: String,        // "higher_is_better", "lower_is_better", "target_range"
    pub target_value: String,
    pub warning_threshold: Option<String>,
    pub critical_threshold: Option<String>,
    pub data_source_query: Option<String>,
    pub evaluation_frequency: String, // "manual", "hourly", "daily", "weekly", "monthly"
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// KPI data point (time-series value)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KpiDataPoint {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub kpi_id: Uuid,
    pub value: String,
    pub recorded_at: DateTime<Utc>,
    pub period_start: Option<chrono::NaiveDate>,
    pub period_end: Option<chrono::NaiveDate>,
    pub status: String, // "on_track", "warning", "critical", "no_target"
    pub notes: Option<String>,
    pub recorded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Dashboard definition
/// Oracle Fusion: Analytics > Dashboards
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Option<Uuid>,
    pub is_shared: bool,
    pub is_default: bool,
    pub layout_config: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Dashboard widget (links a KPI to a dashboard with display config)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardWidget {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub kpi_id: Option<Uuid>,
    pub widget_type: String, // "kpi_card", "chart", "table", "gauge", "trend"
    pub title: String,
    pub position_row: i32,
    pub position_col: i32,
    pub width: i32,
    pub height: i32,
    pub display_config: serde_json::Value,
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// KPI Analytics Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KpiDashboardSummary {
    pub total_kpis: i32,
    pub active_kpis: i32,
    pub on_track: i32,
    pub warning: i32,
    pub critical: i32,
    pub no_data: i32,
    pub total_dashboards: i32,
    pub kpis_by_category: serde_json::Value,
    pub recent_values: serde_json::Value,
}

// ============================================================================
// Account Monitor & Balance Inquiry (Oracle Fusion General Ledger)
// ============================================================================

/// Account Group: a user-defined collection of GL accounts to monitor together.
/// Oracle Fusion equivalent: General Ledger > Journals > Account Monitor > Account Groups
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountGroup {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Option<Uuid>,
    pub is_shared: bool,
    pub threshold_warning_pct: Option<String>,
    pub threshold_critical_pct: Option<String>,
    pub comparison_type: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub members: Vec<AccountGroupMember>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single account within an account group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountGroupMember {
    pub id: Uuid,
    pub group_id: Uuid,
    pub account_segment: String,
    pub account_label: Option<String>,
    pub display_order: i32,
    pub include_children: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Point-in-time GL balance snapshot for a monitored account.
/// Oracle Fusion equivalent: Account Monitor balance rows
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceSnapshot {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub account_group_id: Uuid,
    pub member_id: Option<Uuid>,
    pub account_segment: String,
    pub period_name: String,
    pub period_start: chrono::NaiveDate,
    pub period_end: chrono::NaiveDate,
    pub fiscal_year: i32,
    pub period_number: i32,
    pub beginning_balance: String,
    pub total_debits: String,
    pub total_credits: String,
    pub net_activity: String,
    pub ending_balance: String,
    pub journal_entry_count: i32,
    pub comparison_balance: Option<String>,
    pub comparison_period_name: Option<String>,
    pub variance_amount: Option<String>,
    pub variance_pct: Option<String>,
    pub alert_status: String,
    pub snapshot_date: chrono::NaiveDate,
    pub computed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Saved balance inquiry configuration.
/// Oracle Fusion equivalent: General Ledger > Save Balance Inquiry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedBalanceInquiry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub account_segments: serde_json::Value,
    pub period_from: String,
    pub period_to: String,
    pub currency_code: String,
    pub amount_type: String,
    pub include_zero_balances: bool,
    pub comparison_enabled: bool,
    pub comparison_type: Option<String>,
    pub sort_by: String,
    pub sort_direction: String,
    pub is_shared: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Account Monitor dashboard summary.
/// Oracle Fusion equivalent: Account Monitor summary panel
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountMonitorSummary {
    pub total_groups: i32,
    pub active_groups: i32,
    pub total_members: i32,
    pub snapshots_with_warning: i32,
    pub snapshots_with_critical: i32,
    pub snapshots_on_track: i32,
    pub latest_snapshot_date: Option<chrono::NaiveDate>,
    pub recent_alerts: serde_json::Value,
}

// ============================================================================
// Goal Management (Oracle Fusion HCM > Goal Management)
// ============================================================================

/// Goal Library Category: groups library templates.
/// Oracle Fusion equivalent: Goal Library > Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalLibraryCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i32,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Goal Library Template: predefined goal template.
/// Oracle Fusion equivalent: Goal Library > Goal Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalLibraryTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub category_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub goal_type: String,
    pub success_criteria: Option<String>,
    pub target_metric: Option<String>,
    pub target_value: Option<String>,
    pub uom: Option<String>,
    pub suggested_weight: Option<String>,
    pub estimated_duration_days: Option<i32>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Goal Plan: a performance or development period that contains goals.
/// Oracle Fusion equivalent: Goal Management > Goal Plans
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,
    pub review_period_start: chrono::NaiveDate,
    pub review_period_end: chrono::NaiveDate,
    pub goal_creation_deadline: Option<chrono::NaiveDate>,
    pub status: String,
    pub allow_self_goals: bool,
    pub allow_team_goals: bool,
    pub max_weight_sum: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Goal: individual, team, or organizational goal with progress tracking.
/// Oracle Fusion equivalent: Goal Management > Goals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Option<Uuid>,
    pub parent_goal_id: Option<Uuid>,
    pub library_template_id: Option<Uuid>,
    pub code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub goal_type: String,
    pub category: Option<String>,
    pub owner_id: Uuid,
    pub owner_type: String,
    pub assigned_by: Option<Uuid>,
    pub success_criteria: Option<String>,
    pub target_metric: Option<String>,
    pub target_value: Option<String>,
    pub actual_value: Option<String>,
    pub uom: Option<String>,
    pub progress_pct: Option<String>,
    pub weight: Option<String>,
    pub status: String,
    pub priority: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub completed_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Goal Alignment: explicit link between two goals showing how they relate.
/// Oracle Fusion equivalent: Goal Management > Alignments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalAlignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub source_goal_id: Uuid,
    pub aligned_to_goal_id: Uuid,
    pub alignment_type: String,
    pub description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Goal Note: comment or feedback on a goal.
/// Oracle Fusion equivalent: Goal Management > Notes / Check-ins
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalNote {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub goal_id: Uuid,
    pub author_id: Uuid,
    pub note_type: String,
    pub content: String,
    pub visibility: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Goal Management Dashboard Summary.
/// Oracle Fusion equivalent: Goal Management > Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalManagementSummary {
    pub total_goals: i32,
    pub goals_not_started: i32,
    pub goals_in_progress: i32,
    pub goals_on_track: i32,
    pub goals_at_risk: i32,
    pub goals_completed: i32,
    pub goals_cancelled: i32,
    pub avg_progress_pct: Option<String>,
    pub total_plans: i32,
    pub active_plans: i32,
    pub total_alignments: i32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Landed Cost Management (Oracle Fusion SCM > Landed Cost Management)
// ═══════════════════════════════════════════════════════════════════════════════

/// Landed Cost Template
/// Oracle Fusion: SCM > Landed Cost Management > Cost Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Component (e.g., Freight, Insurance, Customs Duty)
/// Oracle Fusion: SCM > Landed Cost Management > Cost Components
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostComponent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub template_id: Option<Uuid>,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub cost_type: String,
    pub allocation_basis: String,
    pub default_rate: Option<String>,
    pub rate_uom: Option<String>,
    pub expense_account: Option<String>,
    pub is_taxable: bool,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Charge Header
/// Oracle Fusion: SCM > Landed Cost Management > Charges
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostCharge {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub charge_number: String,
    pub template_id: Option<Uuid>,
    pub receipt_id: Option<Uuid>,
    pub purchase_order_id: Option<Uuid>,
    pub supplier_id: Option<Uuid>,
    pub supplier_name: Option<String>,
    pub charge_type: String,
    pub charge_date: Option<chrono::NaiveDate>,
    pub total_amount: String,
    pub currency: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Charge Line
/// Oracle Fusion: SCM > Landed Cost Management > Charge Lines
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostChargeLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub charge_id: Uuid,
    pub component_id: Option<Uuid>,
    pub line_number: i32,
    pub receipt_line_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub charge_amount: String,
    pub allocated_amount: String,
    pub allocation_basis: String,
    pub allocation_qty: Option<String>,
    pub allocation_value: Option<String>,
    pub expense_account: Option<String>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Allocation
/// Oracle Fusion: SCM > Landed Cost Management > Allocations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostAllocation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub charge_id: Uuid,
    pub charge_line_id: Uuid,
    pub receipt_id: Option<Uuid>,
    pub receipt_line_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub allocated_amount: String,
    pub allocation_basis: String,
    pub allocation_basis_value: Option<String>,
    pub total_basis_value: Option<String>,
    pub allocation_pct: Option<String>,
    pub unit_landed_cost: Option<String>,
    pub original_unit_cost: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Simulation
/// Oracle Fusion: SCM > Landed Cost Management > Simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostSimulation {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub simulation_number: String,
    pub template_id: Option<Uuid>,
    pub purchase_order_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub item_code: Option<String>,
    pub item_description: Option<String>,
    pub estimated_quantity: String,
    pub unit_price: String,
    pub currency: String,
    pub estimated_charges: serde_json::Value,
    pub estimated_landed_cost: String,
    pub estimated_landed_cost_per_unit: String,
    pub variance_vs_actual: Option<String>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Landed Cost Dashboard Summary
/// Oracle Fusion: SCM > Landed Cost Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LandedCostDashboard {
    pub total_charges: i32,
    pub pending_charges: i32,
    pub allocated_charges: i32,
    pub total_charge_amount: String,
    pub total_allocated_amount: String,
    pub total_simulations: i32,
    pub charges_by_type: serde_json::Value,
    pub recent_charges: serde_json::Value,
    pub top_cost_components: serde_json::Value,
}

// ============================================================================
// Contract Lifecycle Management (Oracle Fusion Enterprise Contracts)
// ============================================================================

/// CLM Contract Type
/// Oracle Fusion: Enterprise Contracts > Contract Types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmContractType {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub contract_category: String,
    pub default_duration_days: Option<i32>,
    pub requires_approval: bool,
    pub is_auto_renew: bool,
    pub risk_scoring_enabled: bool,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Clause (reusable contract clause)
/// Oracle Fusion: Enterprise Contracts > Clause Library
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmClause {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub title: String,
    pub body: String,
    pub clause_type: String,
    pub clause_category: String,
    pub applicability: String,
    pub is_locked: bool,
    pub version: i32,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Template
/// Oracle Fusion: Enterprise Contracts > Contract Templates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmTemplate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub contract_type_id: Option<Uuid>,
    pub default_currency: String,
    pub default_duration_days: Option<i32>,
    pub terms_and_conditions: Option<String>,
    pub is_standard: bool,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Template Clause
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmTemplateClause {
    pub id: Uuid,
    pub template_id: Uuid,
    pub clause_id: Uuid,
    pub section: Option<String>,
    pub display_order: i32,
    pub is_required: bool,
    pub created_at: DateTime<Utc>,
}

/// CLM Contract
/// Oracle Fusion: Enterprise Contracts > Contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmContract {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_number: String,
    pub title: String,
    pub description: Option<String>,
    pub contract_type_id: Option<Uuid>,
    pub template_id: Option<Uuid>,
    pub contract_category: String,
    pub currency: String,
    pub total_value: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub priority: String,
    pub risk_score: Option<i32>,
    pub risk_level: Option<String>,
    pub parent_contract_id: Option<Uuid>,
    pub renewal_type: String,
    pub auto_renew_months: Option<i32>,
    pub renewal_notice_days: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub approved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Party
/// Oracle Fusion: Enterprise Contracts > Contract Parties
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmContractParty {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub party_type: String,
    pub party_role: String,
    pub party_name: String,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub entity_reference: Option<String>,
    pub is_primary: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Clause (instance in a contract)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmContractClause {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub clause_id: Option<Uuid>,
    pub section: Option<String>,
    pub title: String,
    pub body: String,
    pub clause_type: String,
    pub display_order: i32,
    pub is_modified: bool,
    pub original_body: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Milestone
/// Oracle Fusion: Enterprise Contracts > Milestones
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmMilestone {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub milestone_type: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub completed_date: Option<chrono::NaiveDate>,
    pub amount: Option<String>,
    pub currency: String,
    pub status: String,
    pub responsible_party_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Deliverable
/// Oracle Fusion: Enterprise Contracts > Deliverables
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmDeliverable {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub milestone_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub deliverable_type: String,
    pub quantity: String,
    pub unit_of_measure: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub completed_date: Option<chrono::NaiveDate>,
    pub acceptance_date: Option<chrono::NaiveDate>,
    pub amount: Option<String>,
    pub currency: String,
    pub status: String,
    pub accepted_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Amendment
/// Oracle Fusion: Enterprise Contracts > Amendments
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmAmendment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub amendment_number: String,
    pub title: String,
    pub description: Option<String>,
    pub amendment_type: String,
    pub previous_value: Option<String>,
    pub new_value: Option<String>,
    pub effective_date: Option<chrono::NaiveDate>,
    pub status: String,
    pub approved_by: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Contract Risk Assessment
/// Oracle Fusion: Enterprise Contracts > Risk Management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmRisk {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub contract_id: Uuid,
    pub risk_category: String,
    pub risk_description: String,
    pub probability: String,
    pub impact: String,
    pub mitigation_strategy: Option<String>,
    pub residual_risk: Option<String>,
    pub owner_id: Option<Uuid>,
    pub status: String,
    pub metadata: serde_json::Value,
    pub assessed_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CLM Dashboard Summary
/// Oracle Fusion: Enterprise Contracts > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClmDashboard {
    pub total_contracts: i32,
    pub active_contracts: i32,
    pub draft_contracts: i32,
    pub expiring_contracts: i32,
    pub total_contract_value: String,
    pub contracts_by_category: serde_json::Value,
    pub contracts_by_status: serde_json::Value,
    pub high_risk_contracts: i32,
    pub pending_milestones: i32,
    pub pending_deliverables: i32,
    pub pending_amendments: i32,
    pub recent_contracts: serde_json::Value,
}

// ============================================================================
// Succession Planning
// Oracle Fusion: HCM > Succession Management > Succession Plans, Talent Pools,
//   Talent Reviews, Career Paths
// ============================================================================

/// Succession Plan
/// Oracle Fusion: HCM > Succession Management > Succession Plans
/// A plan for a key position identifying backup candidates and their readiness.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessionPlan {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub plan_type: String,              // position, role, key_person
    pub position_id: Option<Uuid>,
    pub position_title: Option<String>,
    pub job_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub current_incumbent_id: Option<Uuid>,
    pub current_incumbent_name: Option<String>,
    pub risk_level: String,             // low, medium, high, critical
    pub urgency: String,                // immediate, short_term, medium_term, long_term
    pub status: String,                 // draft, active, completed, cancelled
    pub effective_date: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Succession Plan Candidate
/// Oracle Fusion: HCM > Succession Management > Plan Candidates
/// A candidate within a succession plan with readiness assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessionCandidate {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub plan_id: Uuid,
    pub person_id: Uuid,
    pub person_name: Option<String>,
    pub employee_number: Option<String>,
    pub readiness: String,              // ready_now, ready_1_2_years, ready_3_5_years, not_ready
    pub ranking: Option<i32>,
    pub performance_rating: Option<String>,  // 1-5 scale or labels
    pub potential_rating: Option<String>,    // 1-5 scale or labels
    pub flight_risk: Option<String>,         // low, medium, high
    pub development_notes: Option<String>,
    pub recommended_actions: Option<String>,
    pub status: String,                 // proposed, approved, rejected, development
    pub metadata: serde_json::Value,
    pub added_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Talent Pool
/// Oracle Fusion: HCM > Succession Management > Talent Pools
/// A named group of high-potential employees tracked for development.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalentPool {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub pool_type: String,              // leadership, technical, high_potential, diversity, custom
    pub owner_id: Option<Uuid>,
    pub max_members: Option<i32>,
    pub status: String,                 // draft, active, archived
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Talent Pool Member
/// Oracle Fusion: HCM > Succession Management > Pool Members
/// A member of a talent pool with their assessment info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalentPoolMember {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub pool_id: Uuid,
    pub person_id: Uuid,
    pub person_name: Option<String>,
    pub performance_rating: Option<String>,
    pub potential_rating: Option<String>,
    pub readiness: String,              // ready_now, ready_1_2_years, ready_3_5_years, not_ready
    pub development_plan: Option<String>,
    pub notes: Option<String>,
    pub added_date: Option<chrono::NaiveDate>,
    pub review_date: Option<chrono::NaiveDate>,
    pub status: String,                 // active, on_hold, removed, graduated
    pub metadata: serde_json::Value,
    pub added_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Talent Review
/// Oracle Fusion: HCM > Succession Management > Talent Review Meetings
/// A formal assessment session where managers evaluate talent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalentReview {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub review_type: String,            // calibration, performance_potential, nine_box, leadership
    pub facilitator_id: Option<Uuid>,
    pub department_id: Option<Uuid>,
    pub review_date: Option<chrono::NaiveDate>,
    pub status: String,                 // scheduled, in_progress, completed, cancelled
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Talent Review Assessment
/// Oracle Fusion: HCM > Succession Management > Review Assessments
/// An individual assessment within a talent review meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TalentReviewAssessment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub review_id: Uuid,
    pub person_id: Uuid,
    pub person_name: Option<String>,
    pub performance_rating: Option<String>,
    pub potential_rating: Option<String>,
    pub nine_box_position: Option<String>,  // star, workhorse, puzzle, solid_citizen, etc.
    pub strengths: Option<String>,
    pub weaknesses: Option<String>,
    pub career_aspiration: Option<String>,
    pub development_needs: Option<String>,
    pub succession_readiness: Option<String>,
    pub assessor_id: Option<Uuid>,
    pub notes: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Career Path
/// Oracle Fusion: HCM > Succession Management > Career Paths
/// A defined progression path between jobs/roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CareerPath {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub path_type: String,              // linear, branching, lattice, dual_track
    pub from_job_id: Option<Uuid>,
    pub from_job_title: Option<String>,
    pub to_job_id: Option<Uuid>,
    pub to_job_title: Option<String>,
    pub typical_duration_months: Option<i32>,
    pub required_competencies: Option<String>,
    pub required_certifications: Option<String>,
    pub development_activities: Option<String>,
    pub status: String,                 // draft, active, archived
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Succession Planning Dashboard Summary
/// Oracle Fusion: HCM > Succession Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessionDashboard {
    pub total_succession_plans: i32,
    pub active_plans: i32,
    pub plans_by_risk: serde_json::Value,
    pub plans_by_urgency: serde_json::Value,
    pub total_candidates: i32,
    pub candidates_by_readiness: serde_json::Value,
    pub total_talent_pools: i32,
    pub total_pool_members: i32,
    pub total_reviews: i32,
    pub total_career_paths: i32,
    pub coverage_pct: Option<String>,
}

// ============================================================================
// Learning Management
// Oracle Fusion: HCM > Learning > Courses, Specializations, Certifications,
//   Learning Paths, Enrollments, Completions, Assignments
// ============================================================================

/// Learning Item
/// Oracle Fusion: HCM > Learning > Learning Items
/// A learning object such as a course, certification, or specialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub title: String,
    pub description: Option<String>,
    pub item_type: String,            // course, certification, specialization, video, assessment, blended
    pub format: String,               // online, classroom, virtual_classroom, self_paced, blended
    pub category: Option<String>,
    pub provider: Option<String>,
    pub duration_hours: Option<f64>,
    pub currency_code: Option<String>,
    pub cost: Option<String>,
    pub credits: Option<String>,
    pub credit_type: Option<String>,  // ceu, cpe, pdu, college_credit, custom
    pub validity_months: Option<i32>, // how long the certification remains valid
    pub recertification_required: bool,
    pub max_enrollments: Option<i32>,
    pub status: String,               // draft, active, inactive, archived
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Category
/// Oracle Fusion: HCM > Learning > Catalog Categories
/// A hierarchical category for organizing learning items.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub display_order: i32,
    pub status: String,               // active, inactive
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Enrollment
/// Oracle Fusion: HCM > Learning > Enrollments
/// A person's enrollment in a learning item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningEnrollment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub learning_item_id: Uuid,
    pub person_id: Uuid,
    pub person_name: Option<String>,
    pub enrollment_type: String,       // self, manager, mandatory, auto_assigned
    pub enrolled_by: Option<Uuid>,
    pub status: String,               // enrolled, in_progress, completed, failed, withdrawn, expired
    pub progress_pct: Option<String>,
    pub score: Option<String>,
    pub enrollment_date: Option<chrono::NaiveDate>,
    pub completion_date: Option<chrono::NaiveDate>,
    pub due_date: Option<chrono::NaiveDate>,
    pub certification_expiry: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Path
/// Oracle Fusion: HCM > Learning > Learning Paths / Curricula
/// A sequence of learning items forming a curriculum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningPath {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub path_type: String,            // sequential, elective, milestone, tiered
    pub target_role: Option<String>,
    pub target_job_id: Option<Uuid>,
    pub estimated_duration_hours: Option<f64>,
    pub total_items: i32,
    pub status: String,               // draft, active, inactive, archived
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Path Item
/// Oracle Fusion: HCM > Learning > Path Steps
/// A single step/item within a learning path.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningPathItem {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub learning_path_id: Uuid,
    pub learning_item_id: Uuid,
    pub sequence_number: i32,
    pub is_required: bool,
    pub milestone_name: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Assignment
/// Oracle Fusion: HCM > Learning > Mandatory Assignments
/// A mandatory learning requirement assigned to a person or group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningAssignment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub learning_item_id: Option<Uuid>,
    pub learning_path_id: Option<Uuid>,
    pub title: String,
    pub description: Option<String>,
    pub assignment_type: String,       // individual, organization, department, job, position
    pub target_id: Option<Uuid>,
    pub assigned_by: Option<Uuid>,
    pub priority: String,             // low, medium, high, critical
    pub due_date: Option<chrono::NaiveDate>,
    pub status: String,               // active, completed, cancelled
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Learning Dashboard Summary
/// Oracle Fusion: HCM > Learning > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningDashboard {
    pub total_learning_items: i32,
    pub active_items: i32,
    pub items_by_type: serde_json::Value,
    pub total_enrollments: i32,
    pub enrollments_by_status: serde_json::Value,
    pub completion_rate: Option<String>,
    pub total_learning_paths: i32,
    pub total_active_assignments: i32,
    pub overdue_enrollments: i32,
    pub avg_score: Option<String>,
}


// ============================================================================
// Joint Venture Management Types
// Oracle Fusion Cloud Financials > Joint Venture Management
// ============================================================================

/// Joint Venture
/// Oracle Fusion: Financials > Joint Venture Management > Joint Ventures
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JointVenture {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_number: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,               // draft, active, on_hold, closed
    pub operator_id: Option<Uuid>,
    pub operator_name: Option<String>,
    pub currency_code: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub accounting_method: String,    // proportional, equity, cost_method
    pub billing_cycle: String,        // monthly, quarterly, semi_annual, annual
    pub cost_cap_amount: Option<String>,
    pub cost_cap_currency: Option<String>,
    pub gl_revenue_account: Option<String>,
    pub gl_cost_account: Option<String>,
    pub gl_billing_account: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Partner
/// Oracle Fusion: Financials > Joint Venture Management > Partners
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JointVenturePartner {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_id: Uuid,
    pub partner_id: Uuid,
    pub partner_name: String,
    pub partner_type: String,         // operator, non_operator, carried_interest
    pub ownership_percentage: String,
    pub revenue_interest_pct: Option<String>,
    pub cost_bearing_pct: Option<String>,
    pub role: String,                 // operator, partner, carried
    pub billing_contact: Option<String>,
    pub billing_email: Option<String>,
    pub billing_address: Option<String>,
    pub effective_from: chrono::NaiveDate,
    pub effective_to: Option<chrono::NaiveDate>,
    pub status: String,               // active, withdrawn, suspended
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// AFE (Authorization for Expenditure)
/// Oracle Fusion: Financials > Joint Venture Management > AFEs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JointVentureAfe {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_id: Uuid,
    pub afe_number: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,               // draft, submitted, approved, rejected, closed
    pub estimated_cost: String,
    pub actual_cost: String,
    pub committed_cost: String,
    pub remaining_budget: String,
    pub currency_code: String,
    pub cost_center: Option<String>,
    pub work_area: Option<String>,
    pub well_name: Option<String>,
    pub requested_by: Option<Uuid>,
    pub requested_at: Option<DateTime<Utc>>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub rejected_reason: Option<String>,
    pub effective_from: Option<chrono::NaiveDate>,
    pub effective_to: Option<chrono::NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Cost Distribution
/// Oracle Fusion: Financials > Joint Venture Management > Cost Distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvCostDistribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_id: Uuid,
    pub distribution_number: String,
    pub afe_id: Option<Uuid>,
    pub description: Option<String>,
    pub status: String,               // draft, posted, reversed
    pub total_amount: String,
    pub currency_code: String,
    pub cost_type: String,            // operating, capital, aba, overhead
    pub distribution_date: chrono::NaiveDate,
    pub gl_posting_date: Option<chrono::NaiveDate>,
    pub gl_posted_at: Option<DateTime<Utc>>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Cost Distribution Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvCostDistributionLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub distribution_id: Uuid,
    pub partner_id: Uuid,
    pub partner_name: Option<String>,
    pub ownership_pct: String,
    pub cost_bearing_pct: String,
    pub distributed_amount: String,
    pub gl_account_code: Option<String>,
    pub line_description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Revenue Distribution
/// Oracle Fusion: Financials > Joint Venture Management > Revenue Distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvRevenueDistribution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_id: Uuid,
    pub distribution_number: String,
    pub description: Option<String>,
    pub status: String,               // draft, posted, reversed
    pub total_amount: String,
    pub currency_code: String,
    pub revenue_type: String,         // sales, royalty, bonus, other
    pub distribution_date: chrono::NaiveDate,
    pub gl_posting_date: Option<chrono::NaiveDate>,
    pub gl_posted_at: Option<DateTime<Utc>>,
    pub source_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub source_number: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Revenue Distribution Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvRevenueDistributionLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub distribution_id: Uuid,
    pub partner_id: Uuid,
    pub partner_name: Option<String>,
    pub revenue_interest_pct: String,
    pub distributed_amount: String,
    pub gl_account_code: Option<String>,
    pub line_description: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Billing (Joint Interest Billing / JIB)
/// Oracle Fusion: Financials > Joint Venture Management > Billings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvBilling {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub venture_id: Uuid,
    pub billing_number: String,
    pub partner_id: Uuid,
    pub partner_name: Option<String>,
    pub billing_type: String,         // jib (cost), revenue, adjustment
    pub status: String,               // draft, submitted, approved, paid, disputed, cancelled
    pub total_amount: String,
    pub tax_amount: String,
    pub total_with_tax: String,
    pub currency_code: String,
    pub billing_period_start: chrono::NaiveDate,
    pub billing_period_end: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
    pub payment_reference: Option<String>,
    pub dispute_reason: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Billing Line
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvBillingLine {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub billing_id: Uuid,
    pub line_number: i32,
    pub cost_distribution_id: Option<Uuid>,
    pub revenue_distribution_id: Option<Uuid>,
    pub description: Option<String>,
    pub cost_type: Option<String>,
    pub amount: String,
    pub ownership_pct: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joint Venture Dashboard Summary
/// Oracle Fusion: Financials > Joint Venture Management > Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JvDashboard {
    pub total_ventures: i32,
    pub active_ventures: i32,
    pub total_partners: i32,
    pub total_cost_distributed: String,
    pub total_revenue_distributed: String,
    pub total_billed: String,
    pub total_collected: String,
    pub outstanding_balance: String,
    pub pending_afes: i32,
    pub ventures_by_status: serde_json::Value,
}

// ============================================================================
// Risk Management & Internal Controls (Oracle Fusion GRC / Advanced Controls)
// ============================================================================

/// Risk Category
/// Oracle Fusion: GRC > Risk Manager > Risk Categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskCategory {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_category_id: Option<Uuid>,
    pub is_active: bool,
    pub sort_order: i32,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Risk Register Entry
/// Oracle Fusion: GRC > Risk Manager > Risk Register
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub risk_number: String,
    pub title: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub risk_source: String,
    pub likelihood: i32,
    pub impact: i32,
    pub risk_score: i32,
    pub risk_level: String,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub business_units: serde_json::Value,
    pub response_strategy: Option<String>,
    pub residual_likelihood: Option<i32>,
    pub residual_impact: Option<i32>,
    pub identified_date: chrono::NaiveDate,
    pub last_assessed_date: Option<chrono::NaiveDate>,
    pub next_review_date: Option<chrono::NaiveDate>,
    pub closed_date: Option<chrono::NaiveDate>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Control Registry Entry
/// Oracle Fusion: GRC > Advanced Controls > Control Registry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlEntry {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub control_number: String,
    pub title: String,
    pub description: Option<String>,
    pub control_type: String,
    pub control_nature: String,
    pub frequency: String,
    pub objective: Option<String>,
    pub test_procedures: Option<String>,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub is_key_control: bool,
    pub effectiveness: String,
    pub status: String,
    pub business_processes: serde_json::Value,
    pub regulatory_frameworks: serde_json::Value,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Risk-Control Mapping
/// Oracle Fusion: GRC > Risk Manager > Risk-Control Associations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskControlMapping {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub risk_id: Uuid,
    pub control_id: Uuid,
    pub mitigation_effectiveness: String,
    pub status: String,
    pub description: Option<String>,
    pub mapped_by: Option<Uuid>,
    pub mapped_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Control Test
/// Oracle Fusion: GRC > Advanced Controls > Control Testing & Certification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlTest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub control_id: Uuid,
    pub test_number: String,
    pub test_plan: String,
    pub test_period_start: chrono::NaiveDate,
    pub test_period_end: chrono::NaiveDate,
    pub tester_id: Option<Uuid>,
    pub tester_name: Option<String>,
    pub result: String,
    pub findings: Option<String>,
    pub deficiency_severity: Option<String>,
    pub evidence_document_ids: serde_json::Value,
    pub sample_size: Option<i32>,
    pub sample_exceptions: Option<i32>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub reviewer_id: Option<Uuid>,
    pub reviewer_name: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_status: String,
    pub status: String,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Risk Issue / Remediation
/// Oracle Fusion: GRC > Issue Management > Remediation Tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskIssue {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub issue_number: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub risk_id: Option<Uuid>,
    pub control_id: Option<Uuid>,
    pub control_test_id: Option<Uuid>,
    pub severity: String,
    pub priority: String,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub owner_name: Option<String>,
    pub remediation_plan: Option<String>,
    pub remediation_due_date: Option<chrono::NaiveDate>,
    pub remediation_completed_date: Option<chrono::NaiveDate>,
    pub root_cause: Option<String>,
    pub corrective_actions: Option<String>,
    pub identified_date: chrono::NaiveDate,
    pub resolved_date: Option<chrono::NaiveDate>,
    pub closed_date: Option<chrono::NaiveDate>,
    pub regulatory_reference: Option<String>,
    pub metadata: serde_json::Value,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Risk Management Dashboard Summary
/// Oracle Fusion: GRC > Risk Dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskDashboard {
    pub total_risks: i32,
    pub open_risks: i32,
    pub mitigated_risks: i32,
    pub accepted_risks: i32,
    pub critical_risks: i32,
    pub high_risks: i32,
    pub medium_risks: i32,
    pub low_risks: i32,
    pub total_controls: i32,
    pub active_controls: i32,
    pub effective_controls: i32,
    pub ineffective_controls: i32,
    pub not_tested_controls: i32,
    pub total_tests: i32,
    pub passed_tests: i32,
    pub failed_tests: i32,
    pub open_issues: i32,
    pub critical_issues: i32,
    pub overdue_remediations: i32,
    pub risks_by_source: serde_json::Value,
    pub risks_by_level: serde_json::Value,
    pub control_effectiveness_summary: serde_json::Value,
}
