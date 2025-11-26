use cloudflare_kv::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Storage {
    pub name: String,
    pub account_id: String,
    pub namespace_id: String,
    pub api_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Config {
    /// Map of storage names to their configurations
    #[serde(default)]
    pub storages: HashMap<String, Storage>,
    /// Name of the currently active storage
    #[serde(default)]
    pub active_storage: Option<String>,
    /// Legacy fields for backwards compatibility
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,
}

impl Config {
    /// Load or create config
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let mut config: Config = serde_json::from_str(&content).unwrap_or_default();

            // Migrate legacy config format to new format if needed
            let was_migrated = config.storages.is_empty()
                && (config.account_id.is_some()
                    || config.namespace_id.is_some()
                    || config.api_token.is_some());

            if was_migrated {
                config.migrate_legacy_format();
                // Auto-save the migrated config
                config.save(path)?;
            }

            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Migrate from legacy single-storage format to multi-storage format
    pub fn migrate_legacy_format(&mut self) {
        if self.storages.is_empty()
            && (self.account_id.is_some()
                || self.namespace_id.is_some()
                || self.api_token.is_some())
        {
            if let (Some(account_id), Some(namespace_id), Some(api_token)) = (
                self.account_id.take(),
                self.namespace_id.take(),
                self.api_token.take(),
            ) {
                let storage = Storage {
                    name: "default".to_string(),
                    account_id,
                    namespace_id,
                    api_token,
                };
                self.storages.insert("default".to_string(), storage);
                self.active_storage = Some("default".to_string());
            }
        }
    }

    /// Save config to file
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(path)?
                .write_all(content.as_bytes())?;
        }

        #[cfg(not(unix))]
        {
            fs::write(path, content)?;
        }

        Ok(())
    }

    /// Get config directory
    pub fn config_dir() -> Result<PathBuf> {
        if let Ok(xdg_dirs) = xdg::BaseDirectories::new() {
            Ok(xdg_dirs.get_config_home())
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            Ok(PathBuf::from(home).join(".config"))
        }
    }

    /// Get default config file path
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        Ok(config_dir.join("cfkv").join("config.json"))
    }

    /// Add a new storage
    pub fn add_storage(
        &mut self,
        name: String,
        account_id: String,
        namespace_id: String,
        api_token: String,
    ) {
        let storage = Storage {
            name: name.clone(),
            account_id,
            namespace_id,
            api_token,
        };
        self.storages.insert(name.clone(), storage);

        // Set as active if it's the first storage
        if self.active_storage.is_none() {
            self.active_storage = Some(name);
        }
    }

    /// Get a storage by name
    pub fn get_storage(&self, name: &str) -> Option<&Storage> {
        self.storages.get(name)
    }

    /// Get the active storage
    pub fn get_active_storage(&self) -> Option<&Storage> {
        self.active_storage
            .as_ref()
            .and_then(|name| self.storages.get(name))
    }

    /// Set the active storage
    pub fn set_active_storage(&mut self, name: String) -> Result<()> {
        if self.storages.contains_key(&name) {
            self.active_storage = Some(name);
            Ok(())
        } else {
            Err(cloudflare_kv::KvError::InvalidConfig(format!(
                "Storage '{}' not found",
                name
            )))
        }
    }

    /// Remove a storage
    pub fn remove_storage(&mut self, name: &str) -> Result<()> {
        if !self.storages.contains_key(name) {
            return Err(cloudflare_kv::KvError::InvalidConfig(format!(
                "Storage '{}' not found",
                name
            )));
        }

        self.storages.remove(name);

        // If the removed storage was active, switch to another one
        if self.active_storage.as_deref() == Some(name) {
            self.active_storage = self.storages.keys().next().cloned();
        }

        Ok(())
    }

    /// List all storage names
    pub fn list_storages(&self) -> Vec<&str> {
        self.storages.keys().map(|k| k.as_str()).collect()
    }

    /// Rename a storage
    pub fn rename_storage(&mut self, old_name: &str, new_name: String) -> Result<()> {
        if let Some(mut storage) = self.storages.remove(old_name) {
            storage.name = new_name.clone();
            self.storages.insert(new_name.clone(), storage);

            // Update active storage if it was the renamed one
            if self.active_storage.as_deref() == Some(old_name) {
                self.active_storage = Some(new_name);
            }

            Ok(())
        } else {
            Err(cloudflare_kv::KvError::InvalidConfig(format!(
                "Storage '{}' not found",
                old_name
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.storages.is_empty());
        assert_eq!(config.active_storage, None);
    }

    #[test]
    fn test_add_storage() {
        let mut config = Config::default();
        config.add_storage(
            "prod".to_string(),
            "acc123".to_string(),
            "ns456".to_string(),
            "token789".to_string(),
        );

        assert_eq!(config.storages.len(), 1);
        assert_eq!(config.active_storage, Some("prod".to_string()));

        let storage = config.get_storage("prod").unwrap();
        assert_eq!(storage.name, "prod");
        assert_eq!(storage.account_id, "acc123");
    }

    #[test]
    fn test_get_active_storage() {
        let mut config = Config::default();
        config.add_storage(
            "prod".to_string(),
            "acc123".to_string(),
            "ns456".to_string(),
            "token789".to_string(),
        );

        let active = config.get_active_storage().unwrap();
        assert_eq!(active.name, "prod");
    }

    #[test]
    fn test_set_active_storage() {
        let mut config = Config::default();
        config.add_storage(
            "prod".to_string(),
            "acc123".to_string(),
            "ns456".to_string(),
            "token789".to_string(),
        );
        config.add_storage(
            "dev".to_string(),
            "acc999".to_string(),
            "ns999".to_string(),
            "token999".to_string(),
        );

        config.set_active_storage("dev".to_string()).unwrap();
        assert_eq!(config.active_storage, Some("dev".to_string()));
        assert_eq!(config.get_active_storage().unwrap().name, "dev");
    }

    #[test]
    fn test_remove_storage() {
        let mut config = Config::default();
        config.add_storage(
            "prod".to_string(),
            "acc123".to_string(),
            "ns456".to_string(),
            "token789".to_string(),
        );
        config.add_storage(
            "dev".to_string(),
            "acc999".to_string(),
            "ns999".to_string(),
            "token999".to_string(),
        );

        config.set_active_storage("prod".to_string()).unwrap();
        config.remove_storage("prod").unwrap();

        assert_eq!(config.storages.len(), 1);
        assert_eq!(config.active_storage, Some("dev".to_string()));
    }

    #[test]
    fn test_list_storages() {
        let mut config = Config::default();
        config.add_storage(
            "prod".to_string(),
            "acc123".to_string(),
            "ns456".to_string(),
            "token789".to_string(),
        );
        config.add_storage(
            "dev".to_string(),
            "acc999".to_string(),
            "ns999".to_string(),
            "token999".to_string(),
        );

        let storages = config.list_storages();
        assert_eq!(storages.len(), 2);
        assert!(storages.contains(&"prod"));
        assert!(storages.contains(&"dev"));
    }

    #[test]
    fn test_rename_storage() {
        let mut config = Config::default();
        config.add_storage(
            "prod".to_string(),
            "acc123".to_string(),
            "ns456".to_string(),
            "token789".to_string(),
        );

        config.set_active_storage("prod".to_string()).unwrap();
        config
            .rename_storage("prod", "production".to_string())
            .unwrap();

        assert!(config.get_storage("production").is_some());
        assert!(config.get_storage("prod").is_none());
        assert_eq!(config.active_storage, Some("production".to_string()));
    }

    #[test]
    fn test_migration_from_legacy_format() {
        let mut config = Config {
            storages: HashMap::new(),
            active_storage: None,
            account_id: Some("acc123".to_string()),
            namespace_id: Some("ns456".to_string()),
            api_token: Some("token789".to_string()),
        };

        config.migrate_legacy_format();

        assert_eq!(config.storages.len(), 1);
        assert_eq!(config.active_storage, Some("default".to_string()));
        assert!(config.get_storage("default").is_some());
        assert!(config.account_id.is_none());
    }

    #[test]
    fn test_config_serialization_deserialization() {
        let mut config = Config::default();
        config.add_storage(
            "prod".to_string(),
            "acc123".to_string(),
            "ns456".to_string(),
            "token789".to_string(),
        );

        // Serialize
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("prod"));

        // Deserialize
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config.storages.len(), deserialized.storages.len());
    }
}
