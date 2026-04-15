//! Configuration Engine
//! 
//! Hot-reload configuration system for dynamic updates without restarts.

mod engine;
mod loader;

pub use engine::ConfigEngine;
pub use loader::ConfigLoader;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Configuration value
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(untagged)]
#[non_exhaustive]
pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
    #[default]
    Null,
}

/// Configuration entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigEntry {
    pub key: String,
    pub value: ConfigValue,
    pub version: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub updated_by: Option<uuid::Uuid>,
}

/// Configuration change event
#[derive(Debug, Clone)]
pub struct ConfigChange {
    pub key: String,
    pub old_value: Option<ConfigValue>,
    pub new_value: ConfigValue,
}

/// Watcher for configuration changes
pub trait ConfigWatcher: Send + Sync {
    fn on_change(&self, change: ConfigChange);
}

/// Configuration watcher that notifies subscribers
pub struct ConfigWatcherRegistry {
    watchers: RwLock<HashMap<String, Vec<Box<dyn ConfigWatcher>>>>,
}

impl ConfigWatcherRegistry {
    pub fn new() -> Self {
        Self {
            watchers: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a watcher for a config key pattern
    pub async fn register(&self, pattern: &str, watcher: Box<dyn ConfigWatcher>) {
        let mut watchers = self.watchers.write().await;
        watchers
            .entry(pattern.to_string())
            .or_insert_with(Vec::new)
            .push(watcher);
    }
    
    /// Notify watchers of a change
    pub async fn notify(&self, change: &ConfigChange) {
        let watchers = self.watchers.read().await;
        
        for (pattern, watchers_list) in watchers.iter() {
            if self.matches(pattern, &change.key) {
                for watcher in watchers_list {
                    watcher.on_change(change.clone());
                }
            }
        }
    }
    
    fn matches(&self, pattern: &str, key: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        
        if let Some(prefix) = pattern.strip_suffix(".*") {
            return key.starts_with(prefix) && key.len() > prefix.len();
        }
        
        if pattern.contains("*") {
            // Glob matching
            let parts: Vec<&str> = pattern.split('*').collect();
            let mut pos = 0;
            for part in parts {
                if let Some(idx) = key[pos..].find(part) {
                    pos += idx + part.len();
                } else {
                    return false;
                }
            }
            return true;
        }
        
        pattern == key
    }
}

impl Default for ConfigWatcherRegistry {
    fn default() -> Self {
        Self::new()
    }
}
