# CF-KV CLI

A command-line interface for managing Cloudflare Workers KV storage. Written in Rust with async/await support.

**📖 Documentation:** For detailed guides, see the [`docs/`](docs/) folder

## Features

- **CRUD Operations** - Get, put, and delete key-value pairs
- **Batch Operations** - Delete multiple keys at once
- **List & Pagination** - Query keys with cursor-based pagination
- **TTL Support** - Set expiration time for keys
- **Metadata** - Store additional metadata with keys
- **Multiple Output Formats** - JSON, YAML, or plain text
- **Configuration Management** - Store credentials securely
- **Debug Logging** - Optional verbose logging for troubleshooting

## Installation

### Prerequisites
- Rust 1.70 or later
- Cargo

### Build from Source
```bash
git clone <repository-url>
cd cf-kv-cli
cargo build --release
```

The binary will be available at `target/release/cfkv` (or `cfkv.exe` on Windows).

### Install Binary (Recommended)

For easy access from anywhere, install the binary:

```bash
cargo install --path crates/cfkv
```

This installs `cfkv` to `~/.cargo/bin/` (which should be in your PATH if you have Rust installed).

### Using the Binary Without Installation

If you only built the release binary, you can either:

1. **Use the full path**:
   ```bash
   /Users/vv/Projects/Rust/cf-kv-cli/target/release/cfkv --help
   ```

2. **Add to PATH temporarily** (in current shell):
   ```bash
   export PATH="/path/to/cf-kv-cli/target/release:$PATH"
   cfkv --help
   ```

3. **Add to PATH permanently** (edit `~/.zshrc`, `~/.bashrc`, or `~/.bash_profile`):
   ```bash
   export PATH="/path/to/cf-kv-cli/target/release:$PATH"
   ```
   Then reload: `source ~/.zshrc`

4. **Create a symlink**:
   ```bash
   sudo ln -s /path/to/cf-kv-cli/target/release/cfkv /usr/local/bin/cfkv
   ```

## Configuration

### Getting Your Credentials

Before setting up cfkv, you'll need three pieces of information from Cloudflare:

#### 1. API Token
1. Go to https://dash.cloudflare.com/
2. Click your profile icon → **My Profile**
3. Go to **API Tokens** tab
4. Click **Create Token** (use "Edit zone DNS" template or create custom)
5. Copy the token value

#### 2. Account ID
1. Go to https://dash.cloudflare.com/
2. The URL shows: `https://dash.cloudflare.com/YOUR_ACCOUNT_ID`
3. Copy the ID from the URL

Or via API:
```bash
curl -X GET "https://api.cloudflare.com/client/v4/accounts" \
  -H "Authorization: Bearer YOUR_API_TOKEN" \
  -H "Content-Type: application/json"
```

#### 3. Namespace ID (KV Namespace)
1. Go to https://dash.cloudflare.com/
2. Click **Workers & Pages** → **KV**
3. Find your KV namespace and click it
4. Copy the **Namespace ID**

Or check your `wrangler.toml`:
```toml
kv_namespaces = [
  { binding = "MY_KV", id = "abc123xyz789", preview_id = "test123" }
]
```

### Setup
Once you have all three values, configure cfkv:

```bash
cfkv config set-token <YOUR_API_TOKEN>
cfkv config set-account <YOUR_ACCOUNT_ID>
cfkv config set-namespace <YOUR_NAMESPACE_ID>
```

### Configuration File

The configuration is stored at:
- **macOS/Linux**: `~/.config/cfkv/config.json`
- **Windows**: `%APPDATA%\cfkv\config.json`

### View Configuration
```bash
cfkv config show
```

### Reset Configuration
```bash
cfkv config reset
```

### Environment Variables (Alternative)

Instead of storing in config file, use environment variables:

```bash
export CF_API_TOKEN="your-api-token"
export CF_ACCOUNT_ID="your-account-id"
export CF_NAMESPACE_ID="your-namespace-id"

cfkv config show
```

## Multiple Storage Management

For comprehensive storage management documentation, see [**docs/STORAGE_MANAGEMENT.md**](docs/STORAGE_MANAGEMENT.md).

### Overview

cfkv supports managing multiple named storage configurations. This allows you to easily switch between different Cloudflare accounts, namespaces, or environments (production, staging, development, etc.) without having to reconfigure credentials each time.

### Adding a Storage

Add a new named storage with your credentials:

```bash
# Add a production storage
cfkv storage add prod \
  --account-id <ACCOUNT_ID> \
  --namespace-id <NAMESPACE_ID> \
  --api-token <API_TOKEN>

# Add a development storage
cfkv storage add dev \
  --account-id <DEV_ACCOUNT_ID> \
  --namespace-id <DEV_NAMESPACE_ID> \
  --api-token <DEV_API_TOKEN>

# Add a staging storage
cfkv storage add staging \
  --account-id <STAGING_ACCOUNT_ID> \
  --namespace-id <STAGING_NAMESPACE_ID> \
  --api-token <STAGING_API_TOKEN>
```

### Listing All Storages

View all configured storages and see which one is active (marked with `*`):

```bash
cfkv storage list

# Output:
# Available storages:
#
# * prod  (account: abc123, namespace: ns456)
#   dev  (account: def456, namespace: ns789)
#   staging  (account: ghi789, namespace: ns012)
```

With JSON output:

```bash
cfkv --format json storage list
```

### Viewing Current Storage

Display details about the currently active storage:

```bash
cfkv storage current

# Output:
# Current storage: prod
# Account ID: abc123
# Namespace ID: ns456
```

### Switching Between Storages

Switch to a different storage. All subsequent commands will use the new storage:

```bash
# Switch to development storage
cfkv storage switch dev

# Now all commands use the dev storage
cfkv get mykey  # Gets from dev namespace
cfkv put mykey --value "test"  # Puts to dev namespace
```

### Viewing Storage Details

Show details about a specific storage:

```bash
# Show current storage (default)
cfkv storage show

# Show details of a specific storage
cfkv storage show --name prod
```

### Renaming a Storage

Rename a storage configuration:

```bash
cfkv storage rename prod production
```

### Removing a Storage

Remove a storage configuration:

```bash
cfkv storage remove staging
```

If the removed storage was active, cfkv will automatically switch to another available storage.

### Configuration File Format

Storage configurations are saved in your config file:

```json
{
  "storages": {
    "prod": {
      "name": "prod",
      "account_id": "abc123...",
      "namespace_id": "ns456...",
      "api_token": "token789..."
    },
    "dev": {
      "name": "dev",
      "account_id": "def456...",
      "namespace_id": "ns789...",
      "api_token": "token012..."
    }
  },
  "active_storage": "prod"
}
```

### Backwards Compatibility

If you're upgrading from an older version of cfkv that used the legacy single-storage configuration format, your existing configuration will be automatically migrated to the new format on first use:

- Your existing credentials (account_id, namespace_id, api_token) will be migrated to a storage named `default`
- The `default` storage will be set as active
- All subsequent commands will work with the migrated storage

No manual action is required for the migration.

## Usage

### Get a Key
```bash
cfkv get mykey
cfkv get mykey --format json --pretty  # Pretty-printed JSON output
```

### Put a Key
```bash
# With a string value
cfkv put mykey --value "my value"

# With a file
cfkv put mykey --file /path/to/file

# With TTL (time to live in seconds)
cfkv put mykey --value "my value" --ttl 3600

# With metadata
cfkv put mykey --value "my value" --metadata '{"type": "text"}'
```

### Delete a Key
```bash
cfkv delete mykey
```

### List Keys
```bash
# List all keys (default limit: 1000)
cfkv list

# List with custom limit
cfkv list --limit 100

# Pagination using cursor
cfkv list --cursor "next_cursor_value"
```

### Batch Delete
```bash
cfkv batch delete key1 key2 key3
```

### Blog Management

The blog plugin allows you to publish and manage markdown blog posts in Cloudflare KV.

#### Publish a Blog Post
```bash
cfkv blog publish path/to/blog-post.md
```

Blog posts must be markdown files with YAML frontmatter:
```markdown
---
slug: my-blog-post
title: My Blog Post Title
description: A short description of the post
author: Author Name
date: 2025-01-15
cover_image: blog/image.jpg
tags:
  - rust
  - webdev
---

# Your markdown content here

This is the body of your blog post.
```

**Required fields**: slug, title, description, author, date
- `slug`: Post URL identifier (lowercase, numbers, hyphens only)
- `date`: Publication date in YYYY-MM-DD format
- `cover_image`: Optional image path
- `tags`: Optional list of tags

#### List All Blog Posts
```bash
cfkv blog list
cfkv blog list --format json
cfkv blog list --format yaml
```

#### Delete a Blog Post
```bash
cfkv blog delete my-blog-post
```

## Command Line Options

### Global Options
```
--config <PATH>          Path to config file (default: ~/.config/cfkv/config.json)
--account-id <ID>        Cloudflare account ID (overrides config)
--namespace-id <ID>      KV namespace ID (overrides config)
--api-token <TOKEN>      API token (overrides config)
--format <FORMAT>        Output format: text, json, yaml (default: text)
--debug                  Enable debug logging
```

### Get Command
```
--pretty                 Pretty-print JSON output
```

### Put Command
```
--value <VALUE>          String value to store
--file <PATH>            File to store (reads file contents)
--ttl <SECONDS>          Time to live in seconds
--metadata <JSON>        JSON metadata object
```

### List Command
```
--limit <N>              Number of keys to return (default: 1000)
--cursor <CURSOR>        Pagination cursor
--metadata               Include metadata in results
```

## Output Formats

### Text (default)
```bash
$ cfkv get mykey
my value
```

### JSON
```bash
$ cfkv get mykey --format json --pretty
{
  "key": "mykey",
  "value": "my value"
}
```

### YAML
```bash
$ cfkv get mykey --format yaml
key: mykey
value: my value
```

## Project Structure

```
cf-kv-cli/                          # Root workspace
├── Cargo.toml                       # Workspace configuration (no code here)
├── Cargo.lock
├── README.md
├── .gitignore
└── crates/
    ├── cloudflare-kv/              # Core KV client library
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs              # Library exports
    │       ├── client.rs           # HTTP client
    │       ├── auth.rs             # Authentication
    │       ├── types.rs            # Type definitions
    │       ├── error.rs            # Error types
    │       └── batch.rs            # Batch operations
    ├── cfkv/                       # Main CLI application
    │   ├── Cargo.toml
    │   └── src/
    │       ├── main.rs             # Entry point
    │       ├── cli.rs              # CLI command definitions
    │       ├── config.rs           # Configuration management
    │       └── formatter.rs        # Output formatting
    ├── cfkv-blog/                  # Blog plugin crate
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs
    ├── cfkv-config/                # Config utilities plugin crate
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs
    └── cfkv-cache/                 # Cache utilities plugin crate
        ├── Cargo.toml
        └── src/
            └── lib.rs
```

### Workspace Structure Explanation

This project uses a **Rust workspace** pattern:

- **Root `Cargo.toml`**: Defines workspace members and shared dependencies
- **cloudflare-kv**: Core library that can be published separately or used by other projects
- **cfkv**: Binary that uses the core library to provide CLI functionality
- **cfkv-***: Plugin crates that extend the core library for specific use cases

All crates share:
- Same version number
- Shared dependency versions
- Single `Cargo.lock` file

## Development

### Building
```bash
cargo build                    # Debug build
cargo build --release         # Optimized release build
```

### Testing
```bash
cargo test
```

### Running with Arguments
```bash
cargo run -p cfkv -- get mykey
cargo run -p cfkv -- put mykey --value "test"
```

### Debug Logging
```bash
cargo run -p cfkv -- --debug get mykey
```

## Roadmap / TODO

- [ ] Batch import from JSON/YAML files
- [ ] Batch export to files
- [ ] Namespace management commands
- [ ] Interactive REPL mode
- [ ] Configuration profiles for multiple accounts
- [ ] Key filtering and search
- [ ] Performance metrics and statistics

## Dependencies

Key dependencies (from `Cargo.toml`):
- `tokio` - Async runtime
- `reqwest` - HTTP client
- `clap` - CLI argument parsing
- `serde` / `serde_json` / `serde_yaml` - Serialization
- `thiserror` - Error handling
- `tracing` / `tracing-subscriber` - Logging

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## Support

For issues or questions:
1. Check the [Cloudflare KV documentation](https://developers.cloudflare.com/workers/runtime-apis/kv/)
2. Open an issue on the repository
3. Review the debug output with `--debug` flag
