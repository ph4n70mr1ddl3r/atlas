//! API client for communicating with Atlas backend

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// API base URL
fn api_base() -> String {
    std::env!("API_URL").to_string()
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    #[serde(rename = "meta")]
    pub meta: Option<Value>,
}

/// Error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

/// Fetch records for an entity
pub async fn fetch_records(entity: &str, _token: &str) -> Result<Value, String> {
    // In a real implementation, this would use gloo-net or reqwasm
    // For now, return empty data
    Ok(serde_json::json!({
        "data": [],
        "meta": {"total": 0, "offset": 0, "limit": 20, "has_more": false}
    }))
}

/// Fetch a single record
pub async fn fetch_record(entity: &str, id: &str, _token: &str) -> Result<Value, String> {
    Ok(serde_json::json!({}))
}

/// Create a record
pub async fn create_record(entity: &str, data: &Value, _token: &str) -> Result<Value, String> {
    Ok(serde_json::json!({}))
}

/// Update a record
pub async fn update_record(entity: &str, id: &str, data: &Value, _token: &str) -> Result<Value, String> {
    Ok(serde_json::json!({}))
}

/// Delete a record
pub async fn delete_record(entity: &str, id: &str, _token: &str) -> Result<(), String> {
    Ok(())
}

/// Execute a workflow action
pub async fn execute_action(entity: &str, id: &str, action: &str, _token: &str) -> Result<Value, String> {
    Ok(serde_json::json!({"success": true}))
}

/// Fetch entity schema
pub async fn fetch_schema(entity: &str, _token: &str) -> Result<Value, String> {
    Ok(serde_json::json!({}))
}
