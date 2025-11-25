# CF-KV CLI

A command-line interface for managing Cloudflare Workers KV storage. Written in Rust with async/await support.

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

### Setup
Before using the CLI, configure your Cloudflare credentials:

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
