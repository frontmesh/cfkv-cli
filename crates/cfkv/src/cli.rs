use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cfkv",
    version = "0.1.0",
    about = "A general-purpose CLI for Cloudflare KV",
    long_about = "cfkv is a comprehensive CLI tool for managing Cloudflare Workers KV storage with support for interactive and scriptable operations."
)]
pub struct Cli {
    /// Account ID for Cloudflare API
    #[arg(long, env = "CF_ACCOUNT_ID")]
    pub account_id: Option<String>,

    /// Namespace ID for KV storage
    #[arg(long, env = "CF_NAMESPACE_ID")]
    pub namespace_id: Option<String>,

    /// API token for authentication
    #[arg(long, env = "CF_API_TOKEN")]
    pub api_token: Option<String>,

    /// Config file path
    #[arg(long, env = "CF_KV_CONFIG")]
    pub config: Option<PathBuf>,

    /// Output format (json, yaml, text)
    #[arg(short, long, default_value = "text")]
    pub format: String,

    /// Enable debug logging
    #[arg(short, long)]
    pub debug: bool,

    /// Use local KV instance (wrangler dev)
    #[arg(short, long)]
    pub local: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Get a value by key
    Get {
        key: String,
        /// Pretty print output
        #[arg(short, long)]
        pretty: bool,
    },

    /// Put a value with a key
    Put {
        key: String,
        /// Value to store
        #[arg(short, long)]
        value: Option<String>,
        /// Read value from file
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// TTL in seconds
        #[arg(long)]
        ttl: Option<u64>,
        /// Metadata as JSON
        #[arg(long)]
        metadata: Option<String>,
    },

    /// Delete a key
    Delete { key: String },

    /// List all keys
    List {
        /// Number of keys to return
        #[arg(short, long, default_value = "100")]
        limit: u32,
        /// Pagination cursor
        #[arg(long)]
        cursor: Option<String>,
        /// Include metadata
        #[arg(long)]
        metadata: bool,
    },

    /// Batch operations
    Batch {
        #[command(subcommand)]
        command: BatchCommands,
    },

    /// Namespace management
    Namespace {
        #[command(subcommand)]
        command: NamespaceCommands,
    },

    /// Storage management
    Storage {
        #[command(subcommand)]
        command: StorageCommands,
    },

    /// Interactive mode
    Interactive,

    /// Configure authentication
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Blog post management
    Blog {
        #[command(subcommand)]
        command: BlogCommands,
    },
}

#[derive(Subcommand)]
pub enum BatchCommands {
    /// Delete multiple keys
    Delete {
        /// Keys to delete
        keys: Vec<String>,
    },

    /// Put multiple key-value pairs from JSON/YAML file
    Import {
        /// File path
        file: PathBuf,
    },

    /// Export keys to file
    Export {
        /// Output file path
        output: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum NamespaceCommands {
    /// List all namespaces
    List,

    /// Create a new namespace
    Create { name: String },

    /// Switch to a namespace
    Switch { namespace_id: String },

    /// Show current namespace
    Current,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set API token
    SetToken { token: String },

    /// Set account ID
    SetAccount { account_id: String },

    /// Set namespace ID
    SetNamespace { namespace_id: String },

    /// Show current configuration
    Show,

    /// Reset configuration
    Reset,
}

#[derive(Subcommand)]
pub enum StorageCommands {
    /// Add a new storage
    Add {
        /// Storage name
        name: String,
        /// Account ID
        #[arg(short = 'a', long)]
        account_id: String,
        /// Namespace ID
        #[arg(short = 'n', long)]
        namespace_id: String,
        /// API token
        #[arg(short = 't', long)]
        api_token: String,
    },

    /// List all storages
    List,

    /// Show current active storage
    Current,

    /// Switch to a different storage
    Switch {
        /// Storage name to switch to
        name: String,
    },

    /// Remove a storage
    Remove {
        /// Storage name to remove
        name: String,
    },

    /// Rename a storage
    Rename {
        /// Current storage name
        old_name: String,
        /// New storage name
        new_name: String,
    },

    /// Show storage details
    Show {
        /// Storage name (defaults to current storage)
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Export storages to a file
    Export {
        /// Output file path
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Import storages from a file
    Import {
        /// Input file path
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Load storages from environment variables
    LoadEnv,
}

#[derive(Subcommand)]
pub enum BlogCommands {
    /// Publish a blog post from markdown file
    Publish {
        /// Path to markdown file
        file: PathBuf,
    },

    /// List all published blog posts
    List,

    /// Delete a blog post by slug
    Delete {
        /// Post slug
        slug: String,
    },
}
