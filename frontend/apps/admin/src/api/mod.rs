//! API client (reuses dashboard client types)

pub mod client {
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    const API_BASE: &str = "http://localhost:8080";

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

    pub async fn fetch_entity_schema(entity: &str) -> Result<EntitySchema, String> {
        let url = format!("{}/api/v1/schema/{}", API_BASE, entity);
        let response = gloo_net::http::Request::get(&url)
            .header("Authorization", "Bearer demo-token")
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if response.ok() {
            response.json().await.map_err(|e| format!("Parse error: {}", e))
        } else {
            Err(format!("Failed ({})", response.status()))
        }
    }

    pub async fn create_entity_schema(_schema: &EntitySchema) -> Result<(), String> {
        // POST /api/admin/schema
        Ok(())
    }
}
