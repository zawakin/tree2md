# Release Process

This command handles the complete release process for tree2md.

## Steps I will perform:

1. **Determine Version**
   - Review recent changes and suggest appropriate version bump (patch/minor/major)
   - Follow semantic versioning guidelines

2. **Generate Release Description**
   - Analyze commits since last release
   - Create comprehensive release notes with:
     - Key features and improvements
     - Bug fixes
     - Breaking changes (if any)
     - Credits and acknowledgments

3. **Update Version Files**
   - Update `Cargo.toml` version
   - Update `VERSION` constant in `src/main.rs`
   - Update `CHANGELOG.md` with release notes

4. **Validation**
   - Run `cargo fmt -- --check`
   - Run `cargo clippy -- -D warnings`
   - Run `cargo test`
   - Run `cargo build --release`
   - Verify version consistency across files

5. **Commit Version Changes**
   ```bash
   git add -A
   git commit -m "chore: bump version to vX.Y.Z

   <release notes here>

   🤖 Generated with [Claude Code](https://claude.ai/code)

   Co-Authored-By: Claude <noreply@anthropic.com>"
   ```

6. **Create and Push Tag**

   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin main
   git push origin vX.Y.Z
   ```

7. **Monitor Release**
   - Watch GitHub Actions workflow
   - Verify successful build for all platforms
   - Confirm publication to crates.io
   - Check GitHub Release page

## What happens after tagging:

GitHub Actions will automatically:
- Build binaries for Linux, macOS, and Windows
- Create GitHub Release with artifacts
- Publish to crates.io
- Generate SHA256 checksums

## Requirements:

- You must be on the `main` branch
- Working directory must be clean
- All tests must pass
- Version must follow semantic versioning (X.Y.Z)
