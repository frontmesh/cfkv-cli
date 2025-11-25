use crate::error::{KvError, Result};
use crate::types::AuthCredentials;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Authentication manager for handling credentials
pub struct AuthManager {
    credentials: Option<AuthCredentials>,
}

impl AuthManager {
    /// Create a new auth manager
    pub fn new() -> Self {
        Self {
            credentials: None,
        }
    }

    /// Set authentication credentials
    pub fn with_credentials(mut self, credentials: AuthCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Load credentials from environment variable
    pub fn from_env(var_name: &str) -> Result<Self> {
        let token = std::env::var(var_name).map_err(|_| {
            KvError::AuthError(format!(
                "Environment variable {} not found. Set your Cloudflare API token.",
                var_name
            ))
        })?;

        Ok(Self {
            credentials: Some(AuthCredentials::token(token)),
        })
    }

    /// Load credentials from a config file
    pub fn from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(KvError::AuthError(format!(
                "Config file not found: {}",
                path.display()
            )));
        }

        let content = fs::read_to_string(path)?;
        let credentials = Self::parse_config(&content)?;

        Ok(Self {
            credentials: Some(credentials),
        })
    }

    /// Parse credentials from config file content
    fn parse_config(content: &str) -> Result<AuthCredentials> {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match key {
                    "token" => return Ok(AuthCredentials::token(value)),
                    "oauth" => return Ok(AuthCredentials::oauth(value)),
                    _ => {}
                }
            }
        }

        Err(KvError::AuthError(
            "No valid credentials found in config file".to_string(),
        ))
    }

    /// Get the current credentials
    pub fn credentials(&self) -> Result<&AuthCredentials> {
        self.credentials
            .as_ref()
            .ok_or_else(|| KvError::AuthError("No credentials configured".to_string()))
    }

    /// Save credentials to a config file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let creds = self.credentials()?;

        let content = match creds {
            AuthCredentials::Token(token) => format!("token = \"{}\"\n", token),
            AuthCredentials::OAuth(token) => format!("oauth = \"{}\"\n", token),
        };

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write with restrictive permissions (Unix: 600)
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
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager() {
        let manager = AuthManager::new();
        assert!(manager.credentials().is_err());
        
        let creds = AuthCredentials::token("test-token");
        let manager = AuthManager::new().with_credentials(creds);
        assert!(manager.credentials().is_ok());
    }

    #[test]
    fn test_parse_config_credentials() {
        let token_config = r#"token = "secret-token""#;
        match AuthManager::parse_config(token_config).unwrap() {
            AuthCredentials::Token(t) => assert_eq!(t, "secret-token"),
            _ => panic!("Expected token"),
        }
        
        let oauth_config = r#"oauth = "oauth-token""#;
        match AuthManager::parse_config(oauth_config).unwrap() {
            AuthCredentials::OAuth(t) => assert_eq!(t, "oauth-token"),
            _ => panic!("Expected oauth"),
        }
    }

    #[test]
    fn test_parse_config_with_comments_and_spacing() {
        let config = r#"
# Comment
token  =  "my-token"
# More comments
"#;
        match AuthManager::parse_config(config).unwrap() {
            AuthCredentials::Token(t) => assert_eq!(t, "my-token"),
            _ => panic!("Expected token"),
        }
    }

    #[test]
    fn test_parse_config_invalid_cases() {
        assert!(AuthManager::parse_config("").is_err());
        assert!(AuthManager::parse_config("invalid = value").is_err());
    }

    #[test]
    fn test_auth_header_formatting() {
        let token = AuthCredentials::token("api-token");
        assert_eq!(token.auth_header(), "Bearer api-token");
        
        let oauth = AuthCredentials::oauth("oauth-token");
        assert_eq!(oauth.auth_header(), "Bearer oauth-token");
    }
}
