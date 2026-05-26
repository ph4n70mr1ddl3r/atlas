use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::types::*;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

