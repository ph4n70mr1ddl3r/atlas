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
