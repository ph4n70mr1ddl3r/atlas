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
        return_type: Box<Self>,
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
        Self::String { max_length: None, pattern: None }
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
    #[must_use] 
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

pub const fn default_true() -> bool { true }

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
    #[must_use] 
    pub const fn handler_name(&self) -> Option<&str> {
        match self {
            Self::InvokeAction { service: _, action } => {
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

