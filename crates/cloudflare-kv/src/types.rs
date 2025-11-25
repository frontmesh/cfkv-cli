use serde::{Deserialize, Serialize};

/// Authentication credentials for Cloudflare API
#[derive(Clone, Debug)]
pub enum AuthCredentials {
    /// API Token authentication
    Token(String),
    /// OAuth token authentication
    OAuth(String),
}

impl AuthCredentials {
    /// Create new API token credentials
    pub fn token(token: impl Into<String>) -> Self {
        Self::Token(token.into())
    }

    /// Create new OAuth credentials
    pub fn oauth(token: impl Into<String>) -> Self {
        Self::OAuth(token.into())
    }

    /// Get authorization header value
    pub fn auth_header(&self) -> String {
        match self {
            Self::Token(token) => format!("Bearer {}", token),
            Self::OAuth(token) => format!("Bearer {}", token),
        }
    }
}

/// Configuration for Cloudflare KV client
#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub account_id: String,
    pub namespace_id: String,
    pub credentials: AuthCredentials,
    pub base_url: String,
}

impl ClientConfig {
    /// Create new client configuration
    pub fn new(
        account_id: impl Into<String>,
        namespace_id: impl Into<String>,
        credentials: AuthCredentials,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            namespace_id: namespace_id.into(),
            credentials,
            base_url: "https://api.cloudflare.com/client/v4".to_string(),
        }
    }

    /// Get KV API endpoint URL
    pub fn kv_endpoint(&self) -> String {
        format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/values",
            self.base_url, self.account_id, self.namespace_id
        )
    }

    /// Get KV list endpoint URL
    pub fn kv_list_endpoint(&self) -> String {
        format!(
            "{}/accounts/{}/storage/kv/namespaces/{}/keys",
            self.base_url, self.account_id, self.namespace_id
        )
    }
}

/// Pagination parameters for list operations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

impl PaginationParams {
    /// Create new pagination parameters
    pub fn new() -> Self {
        Self {
            limit: None,
            cursor: None,
        }
    }

    /// Set limit for results
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set cursor for pagination
    pub fn with_cursor(mut self, cursor: String) -> Self {
        self.cursor = Some(cursor);
        self
    }
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from list operation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListResponse {
    pub keys: Vec<KeyMetadata>,
    pub list_complete: bool,
    pub cursor: Option<String>,
}

/// Metadata for a KV key
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub name: String,
    pub expiration: Option<u64>,
    pub metadata: Option<serde_json::Value>,
}

/// KV pair with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KvPair {
    pub key: String,
    pub value: String,
    pub metadata: Option<serde_json::Value>,
    pub expiration: Option<u64>,
}
