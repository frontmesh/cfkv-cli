# Release Workflow Guide

This guide explains how the automated release process works and how to use it.

## Overview

The project uses GitHub Actions to automatically:
1. Run tests on every push and pull request
2. Build release binaries for multiple platforms
3. Create releases with tagged versions
4. Auto-tag new versions when merged to main

## Workflows

### Test and Release Workflow

**File**: `.github/workflows/test-and-release.yml`

This workflow handles:
- **Testing**: Runs on Ubuntu, macOS, and Windows
- **Building**: Creates binaries for Linux, macOS (Intel & ARM), and Windows
- **Auto-tagging**: Automatically creates tags on main branch
- **Releasing**: Creates GitHub releases with downloadable binaries

**Triggers**:
- Push to `main` branch
- Push to `feature/**` branches
- Push of version tags (v*)
- Pull requests to `main`

### PR Checks Workflow

**File**: `.github/workflows/pr-checks.yml`

This workflow runs on pull requests and verifies:
- Code formatting (cargo fmt)
- Linting (cargo clippy)
- Tests on all platforms
- Release build success

## How to Make a Release

### Automatic Release (Recommended)

1. **Update the version** in `Cargo.toml`:
   ```toml
   [workspace.package]
   version = "0.2.0"  # Update this
   ```

2. **Commit and push to main**:
   ```bash
   git add Cargo.toml
   git commit -m "chore: bump version to 0.2.0"
   git push origin main
   ```

3. **GitHub Actions does the rest**:
   - Auto-release job detects new version
   - Creates tag `v0.2.0`
   - Build job compiles for all platforms
   - Create-release job makes GitHub Release
   - All binaries available on Releases page

### Manual Release (Alternative)

If you prefer to create tags manually:

```bash
# Update version
sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml

# Create tag
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin main
git push origin v0.2.0
```

## Release Artifacts

When a release is created, the following binaries are built:

- **Linux x86_64**: `cfkv-linux-x86_64.tar.gz`
- **macOS x86_64**: `cfkv-macos-x86_64.tar.gz`
- **macOS ARM64**: `cfkv-macos-aarch64.tar.gz`
- **Windows x86_64**: `cfkv-windows-x86_64.exe`

All are available on the [Releases page](../../releases).

## CI/CD Pipeline Steps

### When you push to a feature branch:

1. PR checks run (format, lint, tests, build)
2. If all pass, you can create a pull request
3. Maintainer reviews and merges to main

### When you merge to main:

1. Test job runs on all platforms
2. Auto-release job checks Cargo.toml version
3. If new version detected, creates tag
4. Tag creation triggers build job
5. Build compiles for all platforms
6. Build completion triggers release job
7. Release job creates GitHub Release with artifacts

### When you push a tag manually:

1. Test job runs
2. Build job creates binaries
3. Release job creates GitHub Release

## Checking Release Status

### View workflow runs:

Go to: **Actions** tab in GitHub repository

Click on **Test and Release** to see:
- Test results
- Build status
- Release creation status

### View releases:

Go to: **Releases** tab in GitHub repository

See all released versions and download binaries.

## Troubleshooting

### Release not being created

**Check**:
1. Did you update Cargo.toml version?
2. Is the new version different from existing tags?
3. Check GitHub Actions logs for auto-release job

**Fix**:
- Verify version bump: `grep "version = " Cargo.toml`
- Check existing tags: `git tag`
- Manually create tag if needed: `git tag v0.2.0 && git push origin v0.2.0`

### Build fails on certain platform

**Check**:
1. View build logs in GitHub Actions
2. Try reproducing locally

**Fix**:
- For Windows issues, check line endings
- For macOS ARM64, ensure target is supported
- For Linux, check for hardcoded Unix assumptions

### Tests failing in CI but passing locally

**Check**:
1. Run same test locally: `cargo test --all`
2. Check GitHub Actions logs for specific error
3. Compare OS environment

**Fix**:
- May be OS-specific issue
- Check for timezone dependencies
- Check for race conditions in tests

### GitHub Actions quota exceeded

**Info**:
- Free tier has 2000 minutes/month
- Each test run uses ~5-10 minutes across all platforms

**Fix**:
- Optimize tests to run faster
- Run tests only on relevant branches
- Use workflow conditions to skip unnecessary runs

## Configuration

### Version Format

The version should follow [Semantic Versioning](https://semver.org/):
- `MAJOR.MINOR.PATCH`
- Examples: `0.1.0`, `1.0.0`, `1.2.3`

### Tags

Tags should follow the pattern `v*`:
- `v0.1.0`
- `v1.0.0`
- `v1.2.3`

The workflow automatically handles this when using auto-release.

## Advanced Usage

### Adding new platforms

Edit `.github/workflows/test-and-release.yml` and add to build matrix:

```yaml
build:
  strategy:
    matrix:
      include:
        # ... existing entries ...
        - os: ubuntu-latest
          target: aarch64-unknown-linux-gnu
          artifact_name: cfkv
          asset_name: cfkv-linux-aarch64
```

### Testing on development branches

Workflows run on `feature/**` branches, so you can:

1. Create feature branch: `git checkout -b feature/my-feature`
2. Push and see PR checks run
3. Open PR and merge to main when ready

### Disabling auto-release temporarily

If you want to merge without creating a release:

Edit `.github/workflows/test-and-release.yml` and add `if` condition:

```yaml
auto-release:
  if: false  # Temporarily disable
```

Remember to remove before pushing!

## Related Documentation

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Semantic Versioning](https://semver.org/)
- [Rust Release Engineering](https://doc.rust-lang.org/cargo/commands/cargo-publish.html)

## Quick Reference

| Action | Command |
|--------|---------|
| View releases | `gh release list` |
| Download artifact | See Releases page |
| Create manual tag | `git tag v0.2.0 && git push origin v0.2.0` |
| Update version | Edit `Cargo.toml` version field |
| Check workflow status | Go to Actions tab |
| View logs | Click workflow run → Click job |

## Support

For issues with GitHub Actions:

1. Check the Actions tab for error logs
2. Review [GITHUB_ACTIONS.md](./GITHUB_ACTIONS.md) for detailed documentation
3. Verify Cargo.toml format and version
4. Ensure you have write permissions to push tags

## Next Steps

1. Make your changes on a feature branch
2. Create a pull request to main
3. Once merged, version is automatically released
4. Download artifacts from Releases page

That's it! The release process is fully automated.