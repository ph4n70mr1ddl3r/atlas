//! Configuration Engine Implementation

use super::{ConfigValue, ConfigChange, ConfigWatcher, ConfigWatcherRegistry};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;
use once_cell::sync::Lazy;

/// Global config registry
pub static CONFIG_REGISTRY: Lazy<Arc<ConfigEngine>> = Lazy::new(|| {
    Arc::new(ConfigEngine::new())
});

/// Main configuration engine
pub struct ConfigEngine {
    values: RwLock<HashMap<String, ConfigValue>>,
    watcher_registry: Arc<ConfigWatcherRegistry>,
}

impl ConfigEngine {
    pub fn new() -> Self {
        Self {
            values: RwLock::new(HashMap::new()),
            watcher_registry: Arc::new(ConfigWatcherRegistry::new()),
        }
    }
    
    /// Get global config engine
    pub fn global() -> Arc<ConfigEngine> {
        CONFIG_REGISTRY.clone()
    }
    
    /// Get a config value
    pub async fn get(&self, key: &str) -> Option<ConfigValue> {
        let values = self.values.read().await;
        values.get(key).cloned()
    }
    
    /// Set a config value
    pub async fn set(&self, key: &str, value: ConfigValue) {
        let mut values = self.values.write().await;
        let old_value = values.insert(key.to_string(), value.clone());
        
        let change = ConfigChange {
            key: key.to_string(),
            old_value,
            new_value: value,
        };
        
        self.watcher_registry.notify(&change).await;
    }
    
    /// Register a watcher
    pub async fn watch(&self, pattern: &str, watcher: Box<dyn ConfigWatcher>) {
        self.watcher_registry.register(pattern, watcher).await;
    }
    
    /// Get all keys
    pub async fn keys(&self) -> Vec<String> {
        let values = self.values.read().await;
        values.keys().cloned().collect()
    }
    
    /// Remove a config value
    pub async fn remove(&self, key: &str) {
        let mut values = self.values.write().await;
        values.remove(key);
    }
}

impl Default for ConfigEngine {
    fn default() -> Self {
        Self::new()
    }
}
