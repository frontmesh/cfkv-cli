# Multi-Storage Management Guide

## Overview

The cfkv CLI now supports managing multiple named storage configurations, allowing you to easily switch between different Cloudflare accounts, namespaces, or environments without reconfiguring credentials each time.

This is particularly useful when you work with:
- Multiple environments (production, staging, development)
- Multiple Cloudflare accounts
- Different KV namespaces for different projects
- Team collaboration where different team members manage different storages

## Quick Start

### 1. Add Your First Storage

```bash
cfkv storage add prod \
  --account-id YOUR_ACCOUNT_ID \
  --namespace-id YOUR_NAMESPACE_ID \
  --api-token YOUR_API_TOKEN
```

This will be automatically set as active.

### 2. Add More Storages

```bash
cfkv storage add dev \
  --account-id DEV_ACCOUNT_ID \
  --namespace-id DEV_NAMESPACE_ID \
  --api-token DEV_API_TOKEN

cfkv storage add staging \
  --account-id STAGING_ACCOUNT_ID \
  --namespace-id STAGING_NAMESPACE_ID \
  --api-token STAGING_API_TOKEN
```

### 3. List Your Storages

```bash
cfkv storage list
```

Output shows which storage is active (marked with `*`):
```
Available storages:

* prod     (account: abc123, namespace: ns456)
  dev      (account: def456, namespace: ns789)
  staging  (account: ghi789, namespace: ns012)
```

### 4. Switch Between Storages

```bash
# Switch to development
cfkv storage switch dev

# All subsequent commands now use dev storage
cfkv get mykey
cfkv put mykey --value "test"
cfkv list
```

## Storage Commands Reference

### Add a Storage

Add a new named storage configuration:

```bash
cfkv storage add <name> \
  --account-id <ACCOUNT_ID> \
  --namespace-id <NAMESPACE_ID> \
  --api-token <API_TOKEN>
```

**Short flags:**
- `-a, --account-id` - Cloudflare account ID
- `-n, --namespace-id` - KV namespace ID
- `-t, --api-token` - Cloudflare API token

**Example:**
```bash
cfkv storage add myproject-prod \
  -a a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6 \
  -n abc123def456ghi789jkl012 \
  -t v1.0a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p
```

### List All Storages

Display all configured storages:

```bash
cfkv storage list
```

**With JSON format:**
```bash
cfkv --format json storage list
```

Output example:
```json
[
  {
    "name": "prod",
    "account_id": "abc123...",
    "namespace_id": "ns456...",
    "active": true
  },
  {
    "name": "dev",
    "account_id": "def456...",
    "namespace_id": "ns789...",
    "active": false
  }
]
```

**With YAML format:**
```bash
cfkv --format yaml storage list
```

### View Current Storage

Show details about the currently active storage:

```bash
cfkv storage current
```

Output:
```
Current storage: prod
Account ID: abc123...
Namespace ID: ns456...
```

### Switch Active Storage

Switch to a different storage. All subsequent commands will use the new storage:

```bash
cfkv storage switch dev
```

You'll see confirmation:
```
Switched to storage 'dev'
```

### Show Storage Details

Display information about a specific storage:

```bash
# Show current active storage
cfkv storage show

# Show details of a specific storage
cfkv storage show --name prod
```

### Rename a Storage

Rename an existing storage configuration:

```bash
cfkv storage rename prod production
```

The rename will:
- Update the storage name
- Preserve all credentials and settings
- If the renamed storage was active, it remains active with the new name

### Remove a Storage

Delete a storage configuration:

```bash
cfkv storage remove staging
```

When removing a storage:
- The storage and its credentials are permanently deleted
- If it was the active storage, cfkv automatically switches to another available storage
- If it was the last storage, you'll need to add a new one before using other commands

## Configuration Storage

### Configuration File Location

Your storage configurations are saved in:
- **macOS/Linux**: `~/.config/cfkv/config.json`
- **Windows**: `%APPDATA%\cfkv\config.json`

### Configuration File Format

Example config file with multiple storages:

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
    },
    "staging": {
      "name": "staging",
      "account_id": "ghi789...",
      "namespace_id": "ns012...",
      "api_token": "token345..."
    }
  },
  "active_storage": "prod"
}
```

### Manual Configuration

You can manually edit the config file if needed, but it's recommended to use the CLI commands for adding/modifying storages.

## Legacy Configuration Migration

### Upgrading from Older Versions

If you're upgrading from an older version of cfkv that used a single storage configuration:

**Old format:**
```json
{
  "account_id": "abc123...",
  "namespace_id": "ns456...",
  "api_token": "token789..."
}
```

**Automatic migration:**
- Your existing credentials will automatically be migrated to a storage named `default`
- Migration happens on your first storage command
- The `default` storage will be set as active
- No manual action is required

**After migration:**
```json
{
  "storages": {
    "default": {
      "name": "default",
      "account_id": "abc123...",
      "namespace_id": "ns456...",
      "api_token": "token789..."
    }
  },
  "active_storage": "default"
}
```

**Using after migration:**
```bash
# Continue using cfkv normally
cfkv list
cfkv get mykey
cfkv put mykey --value "test"

# Add new storages
cfkv storage add prod -a ... -n ... -t ...

# Switch between them
cfkv storage switch prod
```

## Environment Variables

You can override storage credentials using environment variables:

```bash
export CF_ACCOUNT_ID="override_account"
export CF_NAMESPACE_ID="override_namespace"
export CF_API_TOKEN="override_token"

cfkv get mykey  # Uses environment variables, not stored config
```

This overrides whichever storage is currently active.

## Use Cases

### Multi-Environment Setup

```bash
# Set up three environments
cfkv storage add prod -a prod_acc -n prod_ns -t prod_token
cfkv storage add staging -a staging_acc -n staging_ns -t staging_token
cfkv storage add dev -a dev_acc -n dev_ns -t dev_token

# Test in dev first
cfkv storage switch dev
cfkv put feature-flag --value "false"

# Promote to staging
cfkv storage switch staging
cfkv put feature-flag --value "false"

# Deploy to production
cfkv storage switch prod
cfkv put feature-flag --value "true"
```

### Team Collaboration

```bash
# Each team member can have their own configuration
cfkv storage add alice-dev -a alice_acc -n alice_ns -t alice_token
cfkv storage add bob-dev -a bob_acc -n bob_ns -t bob_token
cfkv storage add shared-prod -a prod_acc -n prod_ns -t prod_token

# Switch to collaborate on different storages
cfkv storage switch alice-dev
# ...work on alice's storage...

cfkv storage switch bob-dev
# ...work on bob's storage...
```

### Multiple Projects

```bash
# Different projects with different namespaces
cfkv storage add website-prod -a acc -n website_ns -t token
cfkv storage add api-prod -a acc -n api_ns -t token
cfkv storage add cdn-prod -a acc -n cdn_ns -t token

# Quickly switch between project storages
cfkv storage switch website-prod
cfkv list  # Lists website KV keys

cfkv storage switch api-prod
cfkv list  # Lists API KV keys
```

## Troubleshooting

### Storage Not Found

If you see "Storage not found" error:
```bash
# Check available storages
cfkv storage list

# Verify the storage name
cfkv storage switch correct-name
```

### No Active Storage

If no storage is active:
```bash
# Add a storage (it becomes active automatically)
cfkv storage add default -a ... -n ... -t ...

# Or switch to existing storage
cfkv storage switch prod
```

### Wrong Storage Active

Always verify the active storage before running commands:
```bash
# Check current active storage
cfkv storage current

# Switch if needed
cfkv storage switch prod
```

### Config File Issues

If the config file is corrupted:
```bash
# Reset configuration (removes all storages)
cfkv config reset

# Re-add your storages
cfkv storage add prod -a ... -n ... -t ...
```

## Tips and Best Practices

1. **Use descriptive names**: Use storage names that clearly indicate the environment or project
   ```bash
   # Good
   cfkv storage add myapp-production
   cfkv storage add myapp-staging
   
   # Less clear
   cfkv storage add prod
   cfkv storage add temp
   ```

2. **Always verify active storage**: Before running commands, especially destructive ones
   ```bash
   cfkv storage current  # Always check first
   cfkv get important-key  # Safe to proceed
   ```

3. **Use environment variables for CI/CD**: In automated environments, use env vars instead of storing credentials
   ```bash
   export CF_ACCOUNT_ID=$GITHUB_ACTION_ACCOUNT_ID
   export CF_NAMESPACE_ID=$GITHUB_ACTION_NAMESPACE_ID
   export CF_API_TOKEN=$GITHUB_ACTION_API_TOKEN
   
   cfkv list  # Uses environment variables
   ```

4. **Backup your config**: Keep a backup of your config file
   ```bash
   cp ~/.config/cfkv/config.json ~/.config/cfkv/config.json.backup
   ```

5. **Remove unused storages**: Clean up old storages you no longer use
   ```bash
   cfkv storage remove old-project
   ```

## Related Commands

- `cfkv config show` - Show legacy configuration (if using old format)
- `cfkv config reset` - Reset all configuration (includes all storages)
- `cfkv --help` - Show all available commands
- `cfkv storage --help` - Show all storage-related commands