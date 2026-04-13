//! Atlas Schema Client
//!
//! Client library for interacting with the Atlas schema engine API.
//! Generates type-safe interfaces from dynamic entity definitions.

use serde::{Deserialize, Serialize};

/// Entity definition from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySchema {
    pub name: String,
    pub label: String,
    pub plural_label: String,
    pub fields: Vec<FieldSchema>,
}

/// Field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub required: bool,
    pub editable: bool,
    pub visible: bool,
}

/// Schema client configuration
pub struct SchemaClientConfig {
    pub api_url: String,
    pub auth_token: Option<String>,
}

impl Default for SchemaClientConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:8080".to_string(),
            auth_token: None,
        }
    }
}

/// Schema client for fetching entity definitions
pub struct SchemaClient {
    config: SchemaClientConfig,
}

impl SchemaClient {
    pub fn new(config: SchemaClientConfig) -> Self {
        Self { config }
    }
    
    /// Get the API URL
    pub fn api_url(&self) -> &str {
        &self.config.api_url
    }
    
    /// Build the entity schema endpoint URL
    pub fn schema_url(&self, entity: &str) -> String {
        format!("{}/api/v1/schema/{}", self.config.api_url, entity)
    }
    
    /// Build the entity form config endpoint URL
    pub fn form_url(&self, entity: &str) -> String {
        format!("{}/api/v1/schema/{}/form", self.config.api_url, entity)
    }
    
    /// Build the entity list config endpoint URL
    pub fn list_url(&self, entity: &str) -> String {
        format!("{}/api/v1/schema/{}/list", self.config.api_url, entity)
    }
}
