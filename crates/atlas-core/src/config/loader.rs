//! Configuration Loader
//! 
//! Loads configuration from various sources (files, database, environment).

use super::ConfigValue;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

/// Configuration source
#[derive(Debug, Clone)]
pub enum ConfigSource {
    File(String),
    Database,
    Environment,
    Default,
}

/// Load configuration from YAML file
#[allow(dead_code)]
pub async fn load_from_yaml(path: &Path) -> Result<HashMap<String, ConfigValue>, String> {
    let content = fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse YAML: {}", e))?;
    
    parse_yaml_value("", &yaml)
        .map_err(|e| format!("Failed to parse config: {}", e))
}

/// Parse YAML value into ConfigValue recursively
fn parse_yaml_value(prefix: &str, value: &serde_yaml::Value) -> Result<HashMap<String, ConfigValue>, String> {
    let mut map = HashMap::new();
    
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            for (k, v) in mapping {
                let key = k.as_str().unwrap_or("");
                let full_key = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{}.{}", prefix, key)
                };
                
                match v {
                    serde_yaml::Value::Mapping(_) | serde_yaml::Value::Sequence(_) => {
                        map.extend(parse_yaml_value(&full_key, v)?);
                    }
                    _ => {
                        map.insert(full_key, yaml_to_config(v));
                    }
                }
            }
        }
        _ => {
            map.insert(prefix.to_string(), yaml_to_config(value));
        }
    }
    
    Ok(map)
}

/// Convert YAML value to ConfigValue
fn yaml_to_config(value: &serde_yaml::Value) -> ConfigValue {
    match value {
        serde_yaml::Value::Null => ConfigValue::Null,
        serde_yaml::Value::Bool(b) => ConfigValue::Boolean(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                ConfigValue::Number(i as f64)
            } else if let Some(f) = n.as_f64() {
                ConfigValue::Number(f)
            } else {
                ConfigValue::Null
            }
        }
        serde_yaml::Value::String(s) => ConfigValue::String(s.clone()),
        serde_yaml::Value::Sequence(arr) => {
            ConfigValue::Array(arr.iter().map(yaml_to_config).collect())
        }
        serde_yaml::Value::Mapping(mapping) => {
            let mut obj = HashMap::new();
            for (k, v) in mapping {
                if let Some(key) = k.as_str() {
                    obj.insert(key.to_string(), yaml_to_config(v));
                }
            }
            ConfigValue::Object(obj)
        }
        _ => ConfigValue::Null,
    }
}

/// Load configuration from environment variables
#[allow(dead_code)]
pub fn load_from_env(prefix: &str) -> HashMap<String, ConfigValue> {
    let mut map = HashMap::new();
    
    for (key, value) in std::env::vars() {
        if key.starts_with(prefix) {
            let config_key = key[prefix.len()..]
                .trim_start_matches('_')
                .to_lowercase()
                .replace('_', ".");
            
            let config_value = if value == "true" {
                ConfigValue::Boolean(true)
            } else if value == "false" {
                ConfigValue::Boolean(false)
            } else if let Ok(num) = value.parse::<f64>() {
                ConfigValue::Number(num)
            } else {
                ConfigValue::String(value)
            };
            
            map.insert(config_key, config_value);
        }
    }
    
    map
}

/// Load configuration from JSON file
#[allow(dead_code)]
pub async fn load_from_json(path: &Path) -> Result<HashMap<String, ConfigValue>, String> {
    let content = fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    flatten_json("", &json)
        .map_err(|e| format!("Failed to parse config: {}", e))
}

/// Flatten nested JSON into dot-notation keys
fn flatten_json(prefix: &str, value: &serde_json::Value) -> Result<HashMap<String, ConfigValue>, String> {
    let mut map = HashMap::new();
    
    match value {
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                
                match val {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        map.extend(flatten_json(&full_key, val)?);
                    }
                    _ => {
                        map.insert(full_key, json_to_config(val));
                    }
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let full_key = format!("{}[{}]", prefix, i);
                match val {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        map.extend(flatten_json(&full_key, val)?);
                    }
                    _ => {
                        map.insert(full_key, json_to_config(val));
                    }
                }
            }
        }
        _ => {
            map.insert(prefix.to_string(), json_to_config(value));
        }
    }
    
    Ok(map)
}

/// Convert JSON value to ConfigValue
fn json_to_config(value: &serde_json::Value) -> ConfigValue {
    match value {
        serde_json::Value::Null => ConfigValue::Null,
        serde_json::Value::Bool(b) => ConfigValue::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                ConfigValue::Number(f)
            } else {
                ConfigValue::Null
            }
        }
        serde_json::Value::String(s) => ConfigValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            ConfigValue::Array(arr.iter().map(json_to_config).collect())
        }
        serde_json::Value::Object(obj) => {
            ConfigValue::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), json_to_config(v)))
                    .collect()
            )
        }
    }
}

/// Configuration loader that can merge multiple sources
#[allow(dead_code)]
pub struct ConfigLoader {
    sources: Vec<(ConfigSource, HashMap<String, ConfigValue>)>,
}

impl ConfigLoader {
    pub fn new() -> Self {
        Self { sources: vec![] }
    }
    
    pub fn add_source(mut self, source: ConfigSource, values: HashMap<String, ConfigValue>) -> Self {
        self.sources.push((source, values));
        self
    }
    
    /// Merge all sources with priority (later sources override earlier)
    pub fn merge(&self) -> HashMap<String, ConfigValue> {
        let mut result = HashMap::new();
        
        for (_, values) in &self.sources {
            for (key, value) in values {
                result.insert(key.clone(), value.clone());
            }
        }
        
        result
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_load_from_env() {
        std::env::set_var("ATLAS_DB_HOST", "localhost");
        std::env::set_var("ATLAS_DB_PORT", "5432");
        
        let config = load_from_env("ATLAS");
        
        assert!(matches!(config.get("db.host"), Some(ConfigValue::String(s)) if s == "localhost"));
        assert!(matches!(config.get("db.port"), Some(ConfigValue::Number(n)) if (*n - 5432.0).abs() < f64::EPSILON));
    }
    
    #[test]
    fn test_config_loader_merge() {
        let loader = ConfigLoader::new()
            .add_source(ConfigSource::Default, {
                let mut m = HashMap::new();
                m.insert("app.name".to_string(), ConfigValue::String("Atlas".to_string()));
                m.insert("app.port".to_string(), ConfigValue::Number(8080.0));
                m
            })
            .add_source(ConfigSource::Environment, {
                let mut m = HashMap::new();
                m.insert("app.port".to_string(), ConfigValue::Number(9000.0));
                m
            });
        
        let merged = loader.merge();
        
        assert!(matches!(merged.get("app.name"), Some(ConfigValue::String(s)) if s == "Atlas"));
        assert!(matches!(merged.get("app.port"), Some(ConfigValue::Number(n)) if (n - 9000.0).abs() < f64::EPSILON));
    }
    
    #[tokio::test]
    async fn test_load_from_json() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_config.json");
        
        tokio::fs::write(&config_path, r#"{
            "app": {
                "name": "Atlas ERP",
                "port": 8080,
                "debug": true
            }
        }"#).await.unwrap();
        
        let config = load_from_json(&config_path).await.unwrap();
        
        assert!(matches!(config.get("app.name"), Some(ConfigValue::String(s)) if s == "Atlas ERP"));
        assert!(matches!(config.get("app.port"), Some(ConfigValue::Number(n)) if (n - 8080.0).abs() < f64::EPSILON));
        assert!(matches!(config.get("app.debug"), Some(ConfigValue::Boolean(true))));
        
        tokio::fs::remove_file(&config_path).await.ok();
    }
}
