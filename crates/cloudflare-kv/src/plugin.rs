//! Plugin system for cfkv
//!
//! This module provides the core plugin interface and registry
//! for domain-specific KV use cases.

use async_trait::async_trait;
use serde_json::Value;

/// Plugin metadata
#[derive(Clone, Debug)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// Core plugin trait that all domain-specific plugins must implement
#[async_trait]
pub trait KvPlugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize the plugin with configuration
    async fn init(&mut self, config: Value) -> Result<(), Box<dyn std::error::Error>>;

    /// Process a value before storing in KV
    async fn pre_store(
        &self,
        key: &str,
        value: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>>;

    /// Process a value after retrieving from KV
    async fn post_retrieve(
        &self,
        key: &str,
        value: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>>;

    /// Validate a key-value pair
    async fn validate(
        &self,
        key: &str,
        value: &[u8],
    ) -> Result<bool, Box<dyn std::error::Error>>;

    /// Get plugin-specific commands
    fn commands(&self) -> Vec<String>;
}

/// Plugin registry
pub struct PluginRegistry {
    plugins: std::collections::HashMap<String, Box<dyn KvPlugin>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: std::collections::HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn KvPlugin>) {
        let name = plugin.metadata().name.clone();
        self.plugins.insert(name, plugin);
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<&Box<dyn KvPlugin>> {
        self.plugins.get(name)
    }

    /// Get a mutable plugin by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn KvPlugin>> {
        self.plugins.get_mut(name)
    }

    /// List all registered plugins
    pub fn list(&self) -> Vec<PluginMetadata> {
        self.plugins
            .values()
            .map(|p| p.metadata())
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
