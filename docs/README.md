# Documentation

This folder contains comprehensive documentation for the cf-kv CLI project.

## Quick Navigation

### For Users

- **[STORAGE_MANAGEMENT.md](./STORAGE_MANAGEMENT.md)** - Complete guide to managing multiple named storage configurations
  - Quick start guide
  - Command reference
  - Use cases and examples
  - Troubleshooting
  - Best practices

- **[RELEASE_WORKFLOW.md](./RELEASE_WORKFLOW.md)** - How to make releases and use the automated CI/CD
  - Automatic vs manual releases
  - Release artifacts
  - CI/CD pipeline explanation
  - Troubleshooting release issues

### For Developers

- **[GITHUB_ACTIONS.md](./GITHUB_ACTIONS.md)** - Detailed GitHub Actions workflow documentation
  - Workflow structure and jobs
  - Build process
  - Customization guide
  - Caching strategy
  - Troubleshooting CI/CD

- **[IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)** - Technical overview of multi-storage feature
  - Architecture changes
  - New data structures
  - Backwards compatibility approach
  - Test coverage
  - File changes summary

## Main Documentation

- **[README.md](../README.md)** - Main project documentation in repository root
  - Installation instructions
  - Configuration setup
  - Usage examples
  - Command line options
  - Output formats

## Document Guide

### STORAGE_MANAGEMENT.md
Start here if you want to:
- Add and manage multiple storage configurations
- Switch between different environments (prod/staging/dev)
- Understand how to organize your KV namespaces
- Learn best practices for team collaboration

### RELEASE_WORKFLOW.md
Start here if you want to:
- Make a new release
- Understand how automatic releases work
- Download release binaries
- Troubleshoot release issues

### GITHUB_ACTIONS.md
Start here if you want to:
- Understand the CI/CD pipeline
- Customize workflows
- Add new build targets
- Modify test configurations

### IMPLEMENTATION_SUMMARY.md
Start here if you want to:
- Understand the multi-storage architecture
- See what changed in the codebase
- Review test coverage
- Understand backwards compatibility

## Getting Started

### New User
1. Read [../README.md](../README.md) - Basic setup and usage
2. Read [STORAGE_MANAGEMENT.md](./STORAGE_MANAGEMENT.md) - Learn about storages
3. Try the quick start examples

### New Contributor
1. Read [../README.md](../README.md) - Project overview
2. Read [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md) - Code structure
3. Read [GITHUB_ACTIONS.md](./GITHUB_ACTIONS.md) - CI/CD pipeline
4. Check [RELEASE_WORKFLOW.md](./RELEASE_WORKFLOW.md) - Release process

### Maintainer
1. Read [RELEASE_WORKFLOW.md](./RELEASE_WORKFLOW.md) - Release procedures
2. Read [GITHUB_ACTIONS.md](./GITHUB_ACTIONS.md) - Workflow customization
3. Read [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md) - Architecture

## Key Features

### Multiple Storage Support
Manage multiple Cloudflare accounts and KV namespaces with named configurations.
→ See [STORAGE_MANAGEMENT.md](./STORAGE_MANAGEMENT.md)

### Automated CI/CD
Tests, builds, and releases are automated with GitHub Actions.
→ See [RELEASE_WORKFLOW.md](./RELEASE_WORKFLOW.md) and [GITHUB_ACTIONS.md](./GITHUB_ACTIONS.md)

### Backwards Compatibility
Legacy single-storage configurations are automatically migrated.
→ See [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md)

## Quick Commands

### View All Storages
```bash
cfkv storage list
```

### Switch Storage
```bash
cfkv storage switch prod
```

### Create Release
```bash
# Update version in Cargo.toml
# Then commit and push to main
# GitHub Actions handles the rest!
```

## FAQ

**Q: How do I manage multiple environments?**
A: Use named storages! See [STORAGE_MANAGEMENT.md](./STORAGE_MANAGEMENT.md#multi-environment-setup)

**Q: How do I make a release?**
A: See [RELEASE_WORKFLOW.md](./RELEASE_WORKFLOW.md#how-to-make-a-release)

**Q: How does the CI/CD work?**
A: See [GITHUB_ACTIONS.md](./GITHUB_ACTIONS.md) for detailed documentation

**Q: Can I upgrade from the old config format?**
A: Yes! It's automatic. See [STORAGE_MANAGEMENT.md](./STORAGE_MANAGEMENT.md#backwards-compatibility)

## Table of Contents

| Document | Purpose | Audience |
|----------|---------|----------|
| README.md | Main documentation | All users |
| STORAGE_MANAGEMENT.md | Storage configuration guide | All users |
| RELEASE_WORKFLOW.md | Release process | Maintainers, users |
| GITHUB_ACTIONS.md | CI/CD details | Developers, maintainers |
| IMPLEMENTATION_SUMMARY.md | Technical architecture | Developers |

## Contributing

When submitting changes:
1. Update relevant documentation
2. Follow the release workflow
3. Tests must pass in GitHub Actions
4. Code must pass linting checks

See the project README for contribution guidelines.

## Support

- For feature requests, see GitHub Issues
- For bugs, file a GitHub Issue with details
- For questions, check the documentation first
- For CI/CD issues, see [GITHUB_ACTIONS.md](./GITHUB_ACTIONS.md#troubleshooting)

## Related Resources

- [Main README](../README.md)
- [GitHub Repository](../../)
- [Releases Page](../../releases)
- [GitHub Actions Runs](../../actions)
- [Cloudflare KV Documentation](https://developers.cloudflare.com/workers/runtime-apis/kv/)