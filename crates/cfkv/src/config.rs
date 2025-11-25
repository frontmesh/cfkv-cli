use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use cloudflare_kv::Result;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub account_id: Option<String>,
    pub namespace_id: Option<String>,
    pub api_token: Option<String>,
}

impl Config {
    /// Load or create config
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content).unwrap_or_default())
        } else {
            Ok(Config::default())
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
            let home = std::env::var("HOME")
                .unwrap_or_else(|_| ".".to_string());
            Ok(PathBuf::from(home).join(".config"))
        }
    }

    /// Get default config file path
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        Ok(config_dir.join("cf-kv").join("config.json"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            account_id: None,
            namespace_id: None,
            api_token: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with(account: Option<&str>, namespace: Option<&str>, token: Option<&str>) -> Config {
        Config {
            account_id: account.map(|s| s.to_string()),
            namespace_id: namespace.map(|s| s.to_string()),
            api_token: token.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config, config_with(None, None, None));
    }

    #[test]
    fn test_config_creation() {
        let config = config_with(Some("acc123"), Some("ns456"), Some("token789"));
        assert_eq!(config.account_id, Some("acc123".to_string()));
        assert_eq!(config.namespace_id, Some("ns456".to_string()));
        assert_eq!(config.api_token, Some("token789".to_string()));
    }

    #[test]
    fn test_config_serialization_deserialization() {
        let config = config_with(Some("id123"), Some("ns456"), Some("token789"));
        
        // Serialize
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("id123"));
        
        // Deserialize
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_clone() {
        let config = config_with(Some("id1"), Some("ns1"), Some("token1"));
        let cloned = config.clone();
        assert_eq!(config, cloned);
    }

    #[test]
    fn test_config_partial_values() {
        let config = config_with(Some("account"), None, None);
        assert!(config.account_id.is_some());
        assert!(config.namespace_id.is_none());
        assert!(config.api_token.is_none());
    }
}
