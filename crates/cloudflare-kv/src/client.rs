use crate::error::{KvError, Result};
use crate::types::{ClientConfig, KeyMetadata, KvPair, ListResponse, PaginationParams};
use reqwest::Client;
use serde_json::json;
use tracing::debug;

/// Cloudflare KV client for KV operations
pub struct KvClient {
    http_client: Client,
    config: ClientConfig,
}

impl KvClient {
    /// Create a new KV client
    pub fn new(config: ClientConfig) -> Self {
        let http_client = Client::new();
        Self {
            http_client,
            config,
        }
    }

    /// Get a value from KV by key
    pub async fn get(&self, key: &str) -> Result<Option<KvPair>> {
        let url = format!("{}/{}", self.config.kv_endpoint(), key);
        debug!("Getting key: {}", key);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", self.config.credentials.auth_header())
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let body = response.text().await?;
                Ok(Some(KvPair {
                    key: key.to_string(),
                    value: body,
                    metadata: None,
                    expiration: None,
                }))
            }
            reqwest::StatusCode::NOT_FOUND => Ok(None),
            status => {
                let body = response.text().await?;
                Err(KvError::RequestFailed(format!(
                    "Failed to get key {}: {} - {}",
                    key, status, body
                )))
            }
        }
    }

    /// Put a value into KV
    pub async fn put(&self, key: &str, value: impl AsRef<[u8]>) -> Result<()> {
        let url = format!("{}/{}", self.config.kv_endpoint(), key);
        debug!("Putting key: {}", key);

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", self.config.credentials.auth_header())
            .body(value.as_ref().to_vec())
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(()),
            status => {
                let body = response.text().await?;
                Err(KvError::RequestFailed(format!(
                    "Failed to put key {}: {} - {}",
                    key, status, body
                )))
            }
        }
    }

    /// Put a value with metadata and expiration
    pub async fn put_with_options(
        &self,
        key: &str,
        value: impl AsRef<[u8]>,
        expiration: Option<u64>,
        metadata: Option<serde_json::Value>,
    ) -> Result<()> {
        let url = format!("{}/{}", self.config.kv_endpoint(), key);
        debug!("Putting key with options: {}", key);

        let mut request = self
            .http_client
            .put(&url)
            .header("Authorization", self.config.credentials.auth_header());

        // Add optional query parameters
        if let Some(exp) = expiration {
            request = request.query(&[("expiration_ttl", exp.to_string())]);
        }

        if let Some(meta) = metadata {
            request = request.header("X-Kv-Metadata", meta.to_string());
        }

        let response = request.body(value.as_ref().to_vec()).send().await?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(()),
            status => {
                let body = response.text().await?;
                Err(KvError::RequestFailed(format!(
                    "Failed to put key {}: {} - {}",
                    key, status, body
                )))
            }
        }
    }

    /// Delete a key from KV
    pub async fn delete(&self, key: &str) -> Result<()> {
        let url = format!("{}/{}", self.config.kv_endpoint(), key);
        debug!("Deleting key: {}", key);

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", self.config.credentials.auth_header())
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK | reqwest::StatusCode::NOT_FOUND => Ok(()),
            status => {
                let body = response.text().await?;
                Err(KvError::RequestFailed(format!(
                    "Failed to delete key {}: {} - {}",
                    key, status, body
                )))
            }
        }
    }

    /// List all keys in the namespace with optional pagination
    pub async fn list(&self, params: Option<PaginationParams>) -> Result<ListResponse> {
        let url = self.config.kv_list_endpoint();
        debug!("Listing keys");

        let mut request = self
            .http_client
            .get(&url)
            .header("Authorization", self.config.credentials.auth_header());

        if let Some(params) = params {
            if let Some(limit) = params.limit {
                request = request.query(&[("limit", limit.to_string())]);
            }
            if let Some(cursor) = params.cursor {
                request = request.query(&[("cursor", cursor)]);
            }
        }

        let response = request.send().await?;

        match response.status() {
            reqwest::StatusCode::OK => {
                let body: serde_json::Value = response.json().await?;
                let result = body
                    .get("result")
                    .ok_or_else(|| KvError::RequestFailed("No result in response".to_string()))?;

                let keys: Vec<KeyMetadata> = result
                    .get("keys")
                    .and_then(|k| serde_json::from_value(k.clone()).ok())
                    .unwrap_or_default();

                let list_complete = result
                    .get("list_complete")
                    .and_then(|lc| lc.as_bool())
                    .unwrap_or(false);

                let cursor = result
                    .get("cursor")
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string());

                Ok(ListResponse {
                    keys,
                    list_complete,
                    cursor,
                })
            }
            status => {
                let body = response.text().await?;
                Err(KvError::RequestFailed(format!(
                    "Failed to list keys: {} - {}",
                    status, body
                )))
            }
        }
    }

    /// Batch delete keys
    pub async fn batch_delete(&self, keys: Vec<&str>) -> Result<()> {
        let url = format!("{}/bulk", self.config.kv_endpoint());
        debug!("Batch deleting {} keys", keys.len());

        let body = json!({
            "keys": keys
        });

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", self.config.credentials.auth_header())
            .json(&body)
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(()),
            status => {
                let body = response.text().await?;
                Err(KvError::RequestFailed(format!(
                    "Failed to batch delete: {} - {}",
                    status, body
                )))
            }
        }
    }

    /// Update client configuration
    pub fn update_config(&mut self, config: ClientConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AuthCredentials;

    fn test_config() -> ClientConfig {
        let creds = AuthCredentials::token("test-token");
        ClientConfig::new("account-id", "namespace-id", creds)
    }

    #[test]
    fn test_client_config_creation() {
        let config = test_config();
        let client = KvClient::new(config.clone());
        assert_eq!(client.config().account_id, "account-id");
        assert_eq!(client.config().namespace_id, "namespace-id");
    }

    #[test]
    fn test_endpoint_urls_remote() {
        let config = test_config();
        let kv_endpoint = config.kv_endpoint();
        let list_endpoint = config.kv_list_endpoint();

        assert!(
            kv_endpoint.contains("accounts/account-id/storage/kv/namespaces/namespace-id/values")
        );
        assert!(kv_endpoint.contains("https://api.cloudflare.com/client/v4"));
        assert!(
            list_endpoint.contains("accounts/account-id/storage/kv/namespaces/namespace-id/keys")
        );
        assert!(list_endpoint.contains("https://api.cloudflare.com/client/v4"));
    }

    #[test]
    fn test_endpoint_urls_local() {
        let creds = AuthCredentials::token("test-token");
        let config = ClientConfig::new("account-id", "namespace-id", creds).with_local(true);
        let kv_endpoint = config.kv_endpoint();
        let list_endpoint = config.kv_list_endpoint();

        assert!(
            kv_endpoint.contains("accounts/account-id/storage/kv/namespaces/namespace-id/values")
        );
        assert!(kv_endpoint.contains("http://localhost:8787"));
        assert!(
            list_endpoint.contains("accounts/account-id/storage/kv/namespaces/namespace-id/keys")
        );
        assert!(list_endpoint.contains("http://localhost:8787"));
    }

    #[test]
    fn test_local_remote_switching() {
        let creds = AuthCredentials::token("test-token");
        let mut config = ClientConfig::new("account-id", "namespace-id", creds);

        // Start with remote
        assert!(!config.is_local);
        assert!(config
            .base_url()
            .contains("https://api.cloudflare.com/client/v4"));

        // Switch to local
        config = config.with_local(true);
        assert!(config.is_local);
        assert!(config.base_url().contains("http://localhost:8787"));

        // Switch back to remote
        config = config.with_local(false);
        assert!(!config.is_local);
        assert!(config
            .base_url()
            .contains("https://api.cloudflare.com/client/v4"));
    }

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::new().with_limit(100);
        assert_eq!(params.limit, Some(100));
        assert_eq!(params.cursor, None);

        let params_with_cursor = params.with_cursor("token".to_string());
        assert_eq!(params_with_cursor.cursor, Some("token".to_string()));
    }

    #[test]
    fn test_kv_pair_and_metadata() {
        let pair = KvPair {
            key: "test-key".to_string(),
            value: "test-value".to_string(),
            metadata: None,
            expiration: None,
        };
        assert_eq!(pair.key, "test-key");

        let metadata = KeyMetadata {
            name: "my-key".to_string(),
            expiration: Some(1234567890),
            metadata: None,
        };
        assert_eq!(metadata.name, "my-key");
    }

    #[test]
    fn test_list_response() {
        let response = ListResponse {
            keys: vec![KeyMetadata {
                name: "key1".to_string(),
                expiration: None,
                metadata: None,
            }],
            list_complete: false,
            cursor: Some("next".to_string()),
        };
        assert_eq!(response.keys.len(), 1);
        assert!(!response.list_complete);
    }

    #[test]
    fn test_client_config_update() {
        let config1 = test_config();
        let mut client = KvClient::new(config1);

        let creds = AuthCredentials::token("new-token");
        let config2 = ClientConfig::new("new-account", "new-namespace", creds);
        client.update_config(config2);

        assert_eq!(client.config().account_id, "new-account");
    }

    #[test]
    fn test_auth_header() {
        let token_creds = AuthCredentials::token("my-token");
        assert_eq!(token_creds.auth_header(), "Bearer my-token");

        let oauth_creds = AuthCredentials::oauth("my-oauth");
        assert_eq!(oauth_creds.auth_header(), "Bearer my-oauth");
    }
}
