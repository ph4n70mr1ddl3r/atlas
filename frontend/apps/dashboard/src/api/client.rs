//! API client for communicating with Atlas backend
//!
//! Provides typed HTTP access to the Atlas Gateway REST API.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============================================
// Types
// ============================================

/// API base URL - defaults to localhost
const API_BASE: &str = "http://localhost:8080";

/// Paginated API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
    pub has_more: bool,
}

/// Generic single-record response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordResponse {
    pub data: Value,
}

/// Entity schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySchema {
    pub name: String,
    pub label: String,
    pub plural_label: String,
    pub table_name: Option<String>,
    pub fields: Vec<FieldSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub label: String,
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default = "default_true")]
    pub editable: bool,
    #[serde(default = "default_true")]
    pub visible: bool,
    #[serde(default)]
    pub is_unique: bool,
    #[serde(default)]
    pub is_read_only: bool,
    pub type_config: Option<Value>,
    pub default_value: Option<Value>,
    pub help_text: Option<String>,
}

fn default_true() -> bool { true }

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub initial_state: String,
    pub states: Vec<WorkflowState>,
    pub transitions: Vec<WorkflowTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub name: String,
    pub label: String,
    #[serde(rename = "type")]
    pub state_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransition {
    pub name: String,
    pub from: String,
    pub to: String,
    pub action: String,
    pub label: Option<String>,
    pub required_role: Option<String>,
}

/// Workflow transition info for a record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionInfo {
    pub available_transitions: Vec<AvailableTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableTransition {
    pub name: String,
    pub action: String,
    pub to_state: String,
    pub label: Option<String>,
}

/// Audit history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub action: String,
    pub old_data: Option<Value>,
    pub new_data: Option<Value>,
    pub changed_by: Option<String>,
    pub changed_at: String,
}

/// Dashboard stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub employees: i64,
    pub customers: i64,
    pub open_orders: i64,
    pub active_projects: i64,
    pub recent_activity: Vec<ActivityEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub entity_type: String,
    pub action: String,
    pub description: String,
    pub timestamp: String,
}

/// Login request/response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
}

/// API error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

// ============================================
// Auth Token Management
// ============================================

/// Get stored auth token from local storage
pub fn get_auth_token() -> Option<String> {
    gloo::utils::window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get("atlas_token").ok().flatten())
}

/// Store auth token in local storage
pub fn set_auth_token(token: &str) {
    if let Ok(Some(storage)) = gloo::utils::window().local_storage() {
        let _ = storage.set("atlas_token", token);
    }
}

/// Clear stored auth token
pub fn clear_auth_token() {
    if let Ok(Some(storage)) = gloo::utils::window().local_storage() {
        let _ = storage.delete("atlas_token");
    }
}

/// Get stored user info
pub fn get_stored_user() -> Option<UserInfo> {
    gloo::utils::window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get("atlas_user").ok().flatten())
        .and_then(|u| serde_json::from_str(&u).ok())
}

/// Store user info
pub fn set_stored_user(user: &UserInfo) {
    if let Ok(Some(storage)) = gloo::utils::window().local_storage() {
        if let Ok(json) = serde_json::to_string(user) {
            let _ = storage.set("atlas_user", &json);
        }
    }
}

/// Clear stored user info
pub fn clear_stored_user() {
    if let Ok(Some(storage)) = gloo::utils::window().local_storage() {
        let _ = storage.delete("atlas_user");
    }
}

// ============================================
// API Functions
// ============================================

/// Login to the system
pub async fn login(username: String, password: String) -> Result<LoginResponse, String> {
    let url = format!("{}/api/v1/auth/login", API_BASE);
    let body = LoginRequest { username, password };

    let response = gloo_net::http::Request::post(&url)
        .json(&body)
        .map_err(|e: gloo_net::Error| e.to_string())?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<LoginResponse>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        Err(format!("Login failed ({}): {}", status, text))
    }
}

/// Fetch entity records with pagination and optional search
pub async fn fetch_records(
    entity: &str,
    offset: i64,
    limit: i64,
    search: Option<&str>,
) -> Result<PaginatedResponse<Value>, String> {
    let mut url = format!("{}/api/v1/{}?offset={}&limit={}", API_BASE, entity, offset, limit);
    if let Some(q) = search {
        url.push_str(&format!("&search={}", urlencoding::encode(q)));
    }

    let mut req = gloo_net::http::Request::get(&url);
    if let Some(token) = get_auth_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let response = req
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<PaginatedResponse<Value>>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else if response.status() == 401 {
        Err("Unauthorized".to_string())
    } else {
        Err(format!("Request failed ({})", response.status()))
    }
}

/// Fetch a single record by ID
pub async fn fetch_record(entity: &str, id: &str) -> Result<Value, String> {
    let url = format!("{}/api/v1/{}/{}", API_BASE, entity, id);

    let mut req = gloo_net::http::Request::get(&url);
    if let Some(token) = get_auth_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let response = req
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<RecordResponse>()
            .await
            .map(|r| r.data)
            .map_err(|e| format!("Parse error: {}", e))
    } else if response.status() == 404 {
        Err("Record not found".to_string())
    } else if response.status() == 401 {
        Err("Unauthorized".to_string())
    } else {
        Err(format!("Request failed ({})", response.status()))
    }
}

/// Create a new record
pub async fn create_record(entity: &str, data: &Value) -> Result<Value, String> {
    let url = format!("{}/api/v1/{}", API_BASE, entity);

    let mut req = gloo_net::http::Request::post(&url);
    if let Some(token) = get_auth_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let response = req
        .json(data)
        .map_err(|e: gloo_net::Error| e.to_string())?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<RecordResponse>()
            .await
            .map(|r| r.data)
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(format!("Create failed: {}", text))
    }
}

/// Update a record
pub async fn update_record(entity: &str, id: &str, data: &Value) -> Result<Value, String> {
    let url = format!("{}/api/v1/{}/{}", API_BASE, entity, id);

    let mut req = gloo_net::http::Request::put(&url);
    if let Some(token) = get_auth_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let response = req
        .json(data)
        .map_err(|e: gloo_net::Error| e.to_string())?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<RecordResponse>()
            .await
            .map(|r| r.data)
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(format!("Update failed: {}", text))
    }
}

/// Delete a record
pub async fn delete_record(entity: &str, id: &str) -> Result<(), String> {
    let url = format!("{}/api/v1/{}/{}", API_BASE, entity, id);

    let mut req = gloo_net::http::Request::delete(&url);
    if let Some(token) = get_auth_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let response = req
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        Ok(())
    } else {
        Err(format!("Delete failed ({})", response.status()))
    }
}

/// Fetch entity schema
pub async fn fetch_entity_schema(entity: &str) -> Result<EntitySchema, String> {
    let url = format!("{}/api/v1/schema/{}", API_BASE, entity);

    let mut req = gloo_net::http::Request::get(&url);
    if let Some(token) = get_auth_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let response = req
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err(format!("Failed to load schema ({})", response.status()))
    }
}

/// Get available workflow transitions for a record
pub async fn fetch_transitions(entity: &str, id: &str) -> Result<TransitionInfo, String> {
    let url = format!("{}/api/v1/{}/{}/transitions", API_BASE, entity, id);

    let mut req = gloo_net::http::Request::get(&url);
    if let Some(token) = get_auth_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let response = req
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        Err(format!("Failed to load transitions ({})", response.status()))
    }
}

/// Execute a workflow action on a record
pub async fn execute_action(entity: &str, id: &str, action: &str) -> Result<Value, String> {
    let url = format!("{}/api/v1/{}/{}/{}", API_BASE, entity, id, action);

    let mut req = gloo_net::http::Request::post(&url);
    if let Some(token) = get_auth_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let response = req
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        response
            .json::<RecordResponse>()
            .await
            .map(|r| r.data)
            .map_err(|e| format!("Parse error: {}", e))
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(format!("Action failed: {}", text))
    }
}

/// Get audit history for a record
pub async fn fetch_record_history(entity: &str, id: &str) -> Result<Vec<AuditEntry>, String> {
    let url = format!("{}/api/v1/{}/{}/history", API_BASE, entity, id);

    let mut req = gloo_net::http::Request::get(&url);
    if let Some(token) = get_auth_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let response = req
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        let wrapper: PaginatedResponse<AuditEntry> = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        Ok(wrapper.data)
    } else {
        Err(format!("Failed to load history ({})", response.status()))
    }
}

/// Fetch dashboard report
pub async fn fetch_dashboard_stats() -> Result<DashboardStats, String> {
    let url = format!("{}/api/v1/reports/dashboard", API_BASE);

    let mut req = gloo_net::http::Request::get(&url);
    if let Some(token) = get_auth_token() {
        req = req.header("Authorization", &format!("Bearer {}", token));
    }
    let response = req
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.ok() {
        let wrapper: RecordResponse = response
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        serde_json::from_value(wrapper.data).map_err(|e| format!("Parse error: {}", e))
    } else {
        Err(format!("Failed to load dashboard ({})", response.status()))
    }
}
