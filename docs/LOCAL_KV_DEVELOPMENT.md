# Local KV Development with cfkv

This guide explains how to use cfkv with Wrangler's local KV instance during development.

## Overview

When you run `wrangler dev`, it starts a local development server that includes a local KV instance. This allows you to test your Cloudflare Workers code locally without hitting the production API.

The new `--local` flag in cfkv makes it easy to switch between your local development KV and remote production KV.

## Prerequisites

- **Wrangler** installed and configured
- **cfkv** installed and configured with your Cloudflare credentials
- A `wrangler.toml` file with KV namespace bindings

## Getting Started

### Step 1: Start Wrangler Dev Server

First, start the local Wrangler development server:

```bash
wrangler dev
```

This will start a local server (typically on `http://localhost:8787`) with a local KV instance running.

**Important:** You must keep this terminal window open. The local KV instance only runs while `wrangler dev` is active.

### Step 2: Configure cfkv

Configure cfkv with your Cloudflare credentials (same as you'd use for production):

```bash
cfkv storage add local \
  --account-id <YOUR_ACCOUNT_ID> \
  --namespace-id <YOUR_NAMESPACE_ID> \
  --api-token <YOUR_API_TOKEN>

# Set this storage as active
cfkv storage switch local
```

### Step 3: Use the --local Flag

Now you can use the `-l` or `--local` flag to operate on your local KV instance:

```bash
# Put a value to LOCAL KV
cfkv put mykey --value "test value" --local

# Get a value from LOCAL KV
cfkv get mykey --local

# List keys in LOCAL KV
cfkv list --local

# Delete a key from LOCAL KV
cfkv delete mykey --local
```

## Usage Patterns

### Pattern 1: Always Use Local During Development

When developing locally with `wrangler dev` running, always add the `--local` flag:

```bash
# Development (local KV)
cfkv put user:123 --value '{"name": "Alice"}' --local
cfkv get user:123 --local

# Production (remote KV) - don't forget to remove --local
cfkv put user:123 --value '{"name": "Alice"}'
cfkv get user:123
```

### Pattern 2: Script with Environment Variable

For scripting, you can use an environment variable to control the flag:

```bash
#!/bin/bash

# Set environment for local or remote
ENV=${1:-local}

if [ "$ENV" = "local" ]; then
  LOCAL_FLAG="--local"
else
  LOCAL_FLAG=""
fi

# Use in commands
cfkv put testkey --value "test" $LOCAL_FLAG
cfkv get testkey $LOCAL_FLAG
```

### Pattern 3: Batch Operations

You can mix local and remote operations in a script:

```bash
#!/bin/bash

# Populate local KV for testing
cfkv put test:user1 --value '{"id": 1}' --local
cfkv put test:user2 --value '{"id": 2}' --local

# After testing, copy to production
cfkv get test:user1 --local  # Get from local
cfkv put user1 --value '{"id": 1}'  # Put to remote (no --local flag)
```

## How It Works

### Endpoint Configuration

The `--local` flag switches between two endpoints:

- **Local** (with `--local` flag): `http://localhost:8787`
  - Used during development with `wrangler dev`
  - No authentication required
  - Fast iteration and testing

- **Remote** (without `--local` flag): `https://api.cloudflare.com/client/v4`
  - Used for production
  - Requires API token authentication
  - Real Cloudflare KV storage

### Local Endpoint Details

The default local endpoint is `http://localhost:8787`, which matches Wrangler's default development server port. This can be customized if needed.

The local instance:
- Persists data for the duration of the `wrangler dev` session
- Resets when you stop and restart `wrangler dev`
- Does not require valid Cloudflare credentials
- Provides the same API interface as production KV

## Common Scenarios

### Scenario 1: Testing Data Persistence

```bash
# Terminal 1: Start wrangler dev
wrangler dev

# Terminal 2: Add test data to local KV
cfkv put counter --value "0" --local
cfkv put config:debug --value "true" --local

# Verify it's there
cfkv get counter --local
cfkv list --local

# Stop wrangler dev and restart it
# The data will be gone (fresh local instance)
```

### Scenario 2: Preparing Fixture Data

```bash
# Create a script to setup fixture data for testing
cat > setup_fixtures.sh << 'EOF'
#!/bin/bash

# Setup fixture data in local KV
cfkv put user:1 --value '{"name":"Alice","role":"admin"}' --local
cfkv put user:2 --value '{"name":"Bob","role":"user"}' --local
cfkv put settings:theme --value "dark" --local
cfkv put settings:language --value "en" --local

echo "Fixtures loaded into local KV"
EOF

chmod +x setup_fixtures.sh

# Run it whenever you need fresh test data
./setup_fixtures.sh
```

### Scenario 3: Development vs Production Workflows

```bash
# Development workflow
wrangler dev &
sleep 2
cfkv put dev:test --value "test" --local
cfkv get dev:test --local

# Production workflow
cfkv put prod:data --value "real data"
cfkv get prod:data
```

## Troubleshooting

### "Connection refused" Error

```
Failed to get key mykey: 404 - ...
```

**Solution:** Make sure `wrangler dev` is running in another terminal. The local KV server must be active.

### Data Not Persisting

```
# Put data
cfkv put mykey --value "test" --local

# Stop wrangler dev and restart it

# Data is gone
cfkv get mykey --local  # Not found
```

**Explanation:** This is expected! Local data is temporary and resets when you restart `wrangler dev`.

### Wrong Endpoint Error

If you see errors like "Invalid credentials" when using `--local`, you might have forgotten the flag:

```bash
# Wrong: This tries to hit production without valid credentials
cfkv put mykey --value "test"

# Right: This hits local development server
cfkv put mykey --value "test" --local
```

### Authentication Issues with Local

If you get authentication errors on local operations, remember that the local endpoint doesn't require credentials. If you're having issues:

1. Verify `wrangler dev` is running
2. Check that it's on `http://localhost:8787` (or your configured port)
3. Try the full URL in debug mode:

```bash
cfkv --debug get mykey --local
```

## Advanced Usage

### Custom Local Port

If your Wrangler dev server is running on a different port, you can currently work around this by modifying your configuration or using environment variables (future enhancement to support this directly).

### Batch Import/Export with Local

```bash
# Export from remote
cfkv batch export backup.json

# Import to local for testing
cfkv batch import backup.json --local

# Import to remote after verification
cfkv batch import backup.json
```

## Best Practices

1. **Always use `--local` during development** - This prevents accidental modifications to production data
2. **Keep `wrangler dev` running** - The local KV server needs to be active
3. **Test with production data** - Export production data and import it locally for realistic testing
4. **Reset frequently** - Restart `wrangler dev` to get a fresh local instance
5. **Separate credentials** - Use different storage profiles for local and production

## See Also

- [Storage Management Guide](./STORAGE_MANAGEMENT.md) - Managing multiple KV configurations
- [Wrangler Documentation](https://developers.cloudflare.com/workers/wrangler/) - Wrangler setup and usage
- [Cloudflare KV Documentation](https://developers.cloudflare.com/workers/runtime-apis/kv/) - KV API reference