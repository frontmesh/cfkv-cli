//! Cloudflare KV Library
//!
//! A type-safe, async Rust library for interacting with Cloudflare Workers KV storage.
//!
//! # Features
//!
//! - Get, put, and delete operations
//! - Batch operations and pagination
//! - Type-safe serialization with serde
//! - API token and OAuth authentication
//!
//! # Example
//!
//! ```ignore
//! use cloudflare_kv::{KvClient, ClientConfig, AuthCredentials};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let creds = AuthCredentials::token("your-api-token");
//!     let config = ClientConfig::new("account-id", "namespace-id", creds);
//!     let client = KvClient::new(config);
//!
//!     client.put("key", "value").await?;
//!     let result = client.get("key").await?;
//!
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod batch;
pub mod client;
pub mod error;
pub mod types;

pub use auth::AuthManager;
pub use batch::{BatchBuilder, PaginatedIterator};
pub use client::KvClient;
pub use error::{KvError, Result};
pub use types::{AuthCredentials, ClientConfig, KeyMetadata, KvPair, ListResponse, PaginationParams};
