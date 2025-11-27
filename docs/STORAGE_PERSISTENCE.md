# Storage Persistence and Configuration

This guide covers how to persist your storage configurations, back them up, and load them across different environments using export/import and environment variables.

## Overview

The cfkv CLI provides three ways to manage storage configurations persistently:

1. **File-based Storage** - Automatically saved to your system's config directory
2. **Export/Import** - Backup and restore configurations to/from JSON files
3. **Environment Variables** - Load configurations from environment variables (useful for CI/CD and containerized environments)

## File-based Storage (Default)

Your configurations are automatically stored in:

- **Unix/Linux/macOS**: `~/.config/cfkv/config.json`
- **Windows**: `%APPDATA%/cfkv/config.json`

This is the default location and is automatically created when you add your first storage.

### Reinstalling the Application

After reinstalling cfkv:
1. Your config file remains in the system config directory
2. Simply reinstall cfkv and your storages will be available
3. No reconfiguration needed!

## Export and Import

Use export/import to backup your configurations or transfer them between machines.

### Exporting Storages

Export all your storages to a JSON file:

```bash
# Export to a file
cfkv storage export --file my-storages-backup.json

# Export to stdout (for piping)
cfkv storage export
```

This creates a JSON file like:

```json
{
  "storages": {
    "production": {
      "name": "production",
      "account_id": "account_xyz",
      "namespace_id": "namespace_123",
      "api_token": "token_abc"
    },
    "staging": {
      "name": "staging",
      "account_id": "account_123",
      "namespace_id": "namespace_456",
      "api_token": "token_def"
    }
  },
  "active_storage": "production"
}
```

### Importing Storages

Import configurations from a JSON file:

```bash
cfkv storage import --file my-storages-backup.json
```

This will:
- Load all storages from the JSON file
- Restore the active storage setting
- Merge with existing storages (doesn't delete existing ones)
- Save to your config file

## Environment Variables

Load storage configurations from environment variables. This is ideal for:
- CI/CD pipelines
- Docker containers
- Secure credential management
- Temporary configurations

### Variable Format

Environment variables follow the pattern:

```
CFKV_STORAGE_<NAME>_<FIELD>=<value>
```

Where:
- `<NAME>` is the storage name (e.g., `PROD`, `STAGING`)
- `<FIELD>` is one of: `ACCOUNT_ID`, `NAMESPACE_ID`, `API_TOKEN`

### Examples

Define storages via environment variables:

```bash
# Production storage
export CFKV_STORAGE_PROD_ACCOUNT_ID="account123"
export CFKV_STORAGE_PROD_NAMESPACE_ID="namespace456"
export CFKV_STORAGE_PROD_API_TOKEN="token789"

# Staging storage
export CFKV_STORAGE_STAGING_ACCOUNT_ID="account999"
export CFKV_STORAGE_STAGING_NAMESPACE_ID="namespace999"
export CFKV_STORAGE_STAGING_API_TOKEN="token999"
```

### Loading from Environment

Load all storages defined in environment variables into your config:

```bash
cfkv storage load-env
```

This will:
- Scan for all `CFKV_STORAGE_*` variables
- Create storage entries for each complete set
- Merge with existing storages
- Save to your config file

View what was loaded:

```bash
cfkv storage list
```

## Use Cases

### Backup and Restore

Back up your storages before upgrading or switching machines:

```bash
# Backup
cfkv storage export --file ~/.config/cfkv/backup.json

# After reinstall
cfkv storage import --file ~/.config/cfkv/backup.json
```

### Docker / Containerized Environment

Set up storages via environment variables in your Docker container:

```dockerfile
FROM rust:latest

# Set environment variables
ENV CFKV_STORAGE_PROD_ACCOUNT_ID=my_account
ENV CFKV_STORAGE_PROD_NAMESPACE_ID=my_namespace
ENV CFKV_STORAGE_PROD_API_TOKEN=my_token

RUN cfkv storage load-env
```

### CI/CD Pipelines

Use secrets in your GitHub Actions, GitLab CI, or similar:

```yaml
# GitHub Actions example
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup cfkv
        run: |
          export CFKV_STORAGE_PROD_ACCOUNT_ID=${{ secrets.CF_ACCOUNT_ID }}
          export CFKV_STORAGE_PROD_NAMESPACE_ID=${{ secrets.CF_NAMESPACE_ID }}
          export CFKV_STORAGE_PROD_API_TOKEN=${{ secrets.CF_API_TOKEN }}
          cfkv storage load-env
```

### Multiple Environments

Manage different configurations for dev, staging, and production:

```bash
# Development
export CFKV_STORAGE_DEV_ACCOUNT_ID="dev_account"
export CFKV_STORAGE_DEV_NAMESPACE_ID="dev_namespace"
export CFKV_STORAGE_DEV_API_TOKEN="dev_token"

# Staging
export CFKV_STORAGE_STAGING_ACCOUNT_ID="staging_account"
export CFKV_STORAGE_STAGING_NAMESPACE_ID="staging_namespace"
export CFKV_STORAGE_STAGING_API_TOKEN="staging_token"

# Load all
cfkv storage load-env

# Switch between them
cfkv storage switch dev
cfkv storage switch staging
```

## Security Considerations

### Protecting Export Files

Export files contain sensitive credentials. Protect them carefully:

```bash
# Make export file readable only by you
chmod 600 my-storages-backup.json

# Don't commit to version control
echo "my-storages-backup.json" >> .gitignore

# Consider encrypting the file
gpg -c my-storages-backup.json
```

### Environment Variables in CI/CD

Use your CI/CD platform's secret management:

- **GitHub**: Use Secrets in repository settings
- **GitLab**: Use CI/CD Variables in project settings
- **Jenkins**: Use credentials plugin
- **Docker**: Use secret management tools

Never hardcode credentials in configuration files or scripts.

### Config File Permissions

On Unix systems, the config file is created with restrictive permissions (mode 0o600):
- Only readable/writable by the owner
- No access for group or others

On Windows, use file permissions and access controls as appropriate.

## Troubleshooting

### No storages found when loading from environment

Check that environment variables are properly set:

```bash
# List all CFKV_STORAGE variables
env | grep CFKV_STORAGE

# Check a specific storage
echo $CFKV_STORAGE_PROD_ACCOUNT_ID
```

All three fields (ACCOUNT_ID, NAMESPACE_ID, API_TOKEN) must be set for a storage to be recognized.

### Import overwrites active storage

Import merges with existing storages but restores the `active_storage` setting from the file. If you want to keep your current active storage:

1. Note the current active storage: `cfkv storage current`
2. Import the file: `cfkv storage import --file backup.json`
3. Switch back if needed: `cfkv storage switch <name>`

### Permissions issues on Unix

If you get permission errors accessing the config file:

```bash
# Fix permissions
chmod 600 ~/.config/cfkv/config.json
```

## Related Commands

- `cfkv storage list` - List all storages
- `cfkv storage add` - Add a new storage
- `cfkv storage switch` - Switch active storage
- `cfkv storage show` - Show storage details
- `cfkv storage current` - Show current active storage