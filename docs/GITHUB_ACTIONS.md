# GitHub Actions Workflows

This project uses GitHub Actions to automate testing, building, and releasing.

## Workflows

### 1. Test and Release (`test-and-release.yml`)

**Triggers:**
- Push to `main` branch
- Push to `feature/**` branches
- Push of version tags (`v*`)
- Pull requests to `main` branch

**Jobs:**

#### Test Job
- Runs on: Ubuntu, macOS, Windows (latest)
- Steps:
  - Install Rust (stable)
  - Cache dependencies
  - Run test suite (debug and release)
  - Check code formatting with `cargo fmt`
  - Lint with `cargo clippy`

#### Build Job
- Runs on: Ubuntu (Linux), macOS (x86_64 and ARM64), Windows
- Triggers: Only on version tag pushes
- Builds release binaries for multiple platforms
- Artifacts uploaded and available as downloads

#### Auto-Release Job
- Runs on: Ubuntu
- Triggers: Only on push to `main` branch
- Steps:
  1. Reads version from `Cargo.toml`
  2. Checks if tag exists
  3. If tag doesn't exist, creates and pushes it
  4. This automatically triggers the build and release job

#### Create Release Job
- Runs on: Ubuntu
- Triggers: Only when a version tag is pushed
- Steps:
  1. Downloads all built binaries from artifacts
  2. Packages Unix binaries as `.tar.gz`
  3. Creates GitHub Release with assets
  4. Auto-generates release notes from commits

### 2. Pull Request Checks (`pr-checks.yml`)

**Triggers:**
- Pull requests to `main` or `feature/**` branches

**Jobs:**

#### Check Job
- Runs on: Ubuntu
- Steps:
  - Check code formatting
  - Run clippy linter with warnings as errors

#### Test Job
- Runs on: Ubuntu, macOS, Windows
- Steps:
  - Run full test suite (debug and release)

#### Build Job
- Runs on: Ubuntu
- Steps:
  - Build release binary
  - Verify binary works (`--version`)

## Release Process

### Automatic Releases

1. **Merge to main**: When you merge a pull request to `main`
2. **Auto-tag**: GitHub Actions reads `Cargo.toml` version and creates a tag if it doesn't exist
3. **Build**: Tag creation triggers the build job, which compiles for all platforms
4. **Release**: Build completion triggers release job, which creates a GitHub Release with binaries

### Manual Releases

If you want to create a release manually:

1. Update version in `Cargo.toml`:
   ```toml
   [workspace.package]
   version = "0.2.0"
   ```

2. Merge to main branch

3. GitHub Actions automatically:
   - Detects the new version
   - Creates tag `v0.2.0`
   - Builds binaries
   - Creates GitHub Release

### Manual Tag Creation (Alternative)

If you want to create a tag manually:

```bash
# Update version in Cargo.toml first
git add Cargo.toml
git commit -m "chore: bump version to 0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin main
git push origin v0.2.0
```

This will trigger the build and release workflows.

## Build Artifacts

When a version tag is created, the following binaries are built and released:

- **Linux**: `cfkv-linux-x86_64.tar.gz`
- **macOS (Intel)**: `cfkv-macos-x86_64.tar.gz`
- **macOS (ARM64)**: `cfkv-macos-aarch64.tar.gz`
- **Windows**: `cfkv-windows-x86_64.exe`

All artifacts are available on the [GitHub Releases](../../releases) page.

## Environment Variables

The workflows use these environment variables:

- `CARGO_TERM_COLOR`: Set to `always` for colored output
- `RUST_BACKTRACE`: Set to `1` for detailed error information

## Caching

All workflows use GitHub's cache action to speed up builds:

- Cargo registry cache
- Cargo git index cache
- Target build directory cache

Caches are keyed by:
- Operating system
- `Cargo.lock` file hash

This ensures cache hits when dependencies haven't changed.

## Requirements

To use these workflows, your repository needs:

1. **Rust**: Installed via `dtolnay/rust-toolchain@stable`
2. **Git**: For tag creation and pushing
3. **GitHub Token**: Automatically provided by GitHub Actions (`GITHUB_TOKEN`)

## Troubleshooting

### Release job doesn't trigger

**Problem**: You pushed a tag but the release job didn't run.

**Solution**:
- Make sure the tag matches the pattern `v*` (e.g., `v0.1.0`, `v1.2.3`)
- Check that you pushed the tag: `git push origin <tag-name>`
- Verify on GitHub Actions page that the job was triggered

### Auto-release doesn't create a tag

**Problem**: You merged to main but no tag was created.

**Solution**:
- Check that the version in `Cargo.toml` is different from existing tags
- Verify that the commit reached the `main` branch
- Check GitHub Actions logs for the `auto-release` job

### Build fails on Windows

**Problem**: Windows build fails while Linux/macOS succeed.

**Solution**:
- Windows uses `msvc` toolchain (installed automatically)
- Most build failures are due to code issues, not Windows-specific
- Check the logs to identify the specific error
- Common issues: Line endings (use `.gitattributes`), path separators

### Test fails on specific OS

**Problem**: Tests pass locally but fail on CI for a specific OS.

**Solution**:
- Re-run the workflow on that specific OS
- Check for platform-specific code paths
- Use `cfg` attributes for OS-specific code
- Run tests locally on the same OS if possible

## Customization

### Adding new targets

To add more build targets, edit `.github/workflows/test-and-release.yml` and add to the `build` job's matrix:

```yaml
- os: ubuntu-latest
  target: aarch64-unknown-linux-gnu
  artifact_name: cfkv
  asset_name: cfkv-linux-aarch64
```

### Changing test matrix

To add or remove OS test targets, modify the `test` job matrix:

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, nightly]  # Add nightly testing
```

### Modifying build steps

To add custom build steps (e.g., running benchmarks), edit the respective workflow file and add steps before or after the `cargo build` command.

## Related Files

- `.github/workflows/test-and-release.yml` - Main workflow
- `.github/workflows/pr-checks.yml` - Pull request workflow
- `Cargo.toml` - Version source for auto-release
- `.gitignore` - Excludes target directory

## See Also

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust on GitHub Actions](https://github.com/actions-rs/meta)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)
```

Now let me commit all these changes: