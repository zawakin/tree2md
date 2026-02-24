# Release Process

This command handles the complete release process for tree2md.

## Prerequisites

- Must be on `main` branch with clean working directory
- All changes go through PRs (per CLAUDE.md) — including version bumps
- Load `/git-workflow` first (per CLAUDE.md)

## Steps I will perform:

1. **Determine Version**
   - Review recent changes and suggest appropriate version bump (patch/minor/major)
   - Follow semantic versioning guidelines
   - Ask user to confirm version

2. **Generate Release Description**
   - Analyze commits since last release (`git log $(git tag --sort=-v:refname | head -1)..HEAD`)
   - Create comprehensive release notes with:
     - Key features and improvements
     - Bug fixes
     - Breaking changes (if any)

3. **Create Release Branch & Update Files**
   ```bash
   mise run git:new release/vX.Y.Z
   ```
   - Update `Cargo.toml` version
   - Run `cargo generate-lockfile` to update `Cargo.lock`
   - Update `CHANGELOG.md` with release notes (including comparison links at bottom)
   - Do NOT update `src/main.rs` VERSION constant (it does not exist)

4. **Validation**
   - Run `mise run verify` (fmt, clippy, test)
   - Run `cargo build --release`
   - Verify `Cargo.lock` is tracked and up to date

5. **Commit, Push, and Create PR**
   ```bash
   git add Cargo.toml Cargo.lock CHANGELOG.md
   git commit -m "chore: bump version to vX.Y.Z

   <brief release summary>

   Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
   git push -u origin release/vX.Y.Z
   gh pr create -a "@me" -t "chore: bump version to vX.Y.Z" --body "..."
   ```

6. **PR Lifecycle**
   ```bash
   mise run git:open-pr -- <pr#>   # background: CI → browser → merge watch → cleanup
   ```
   Wait for PR to be merged.

7. **Create and Push Tag** (after PR merge)
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```

8. **Monitor Release**
   ```bash
   gh run watch <run-id> --exit-status   # run in background
   ```
   - Verify all 3 build jobs pass (linux, macos, windows)
   - Verify GitHub Release is created with artifacts
   - Verify crates.io publish succeeds
   - If crates.io fails: check logs with `gh run view <run-id> --log-failed`

## Troubleshooting: crates.io publish failure

Common causes:
- **Dirty Cargo.lock**: ensure `Cargo.lock` is committed in the repo
- **Token issue**: check `CARGO_REGISTRY_TOKEN` secret in repo settings
- **Already published**: version already exists on crates.io

To retry crates.io publish after fixing:
```bash
# Option A: re-run just the failed job
gh run rerun <run-id> --failed

# Option B: delete tag, fix, re-tag
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
gh release delete vX.Y.Z --yes
# ... fix and merge via PR ...
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin vX.Y.Z
```

## What happens after tagging:

GitHub Actions will automatically:
- Build binaries for Linux, macOS, and Windows
- Create GitHub Release with artifacts
- Publish to crates.io
- Generate SHA256 checksums
