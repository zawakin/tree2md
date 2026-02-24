---
name: release
description: Release workflow — version bump, changelog, tagging, and CI monitoring. Use when preparing a new release.
---

# Release Workflow

Tag-driven release using `mise run release:*` tasks and GitHub Actions.

## Quick Reference

```sh
mise run release:tag              # Create & push git tag from Cargo.toml version
mise run release:watch            # Watch release CI (build + publish)
```

## Release Flow

```
1. Prepare   → branch, version bump, changelog, PR
2. Tag       → mise run release:tag     (after PR merge)
3. Watch     → mise run release:watch   (CI → GitHub Release → crates.io)
```

### Phase 1: Prepare (manual)

1. **Determine version** — review changes since last tag, decide patch/minor/major
   ```sh
   git log $(git tag --sort=-v:refname | head -1)..HEAD --oneline
   ```

2. **Create release branch**
   ```sh
   mise run git:new release/vX.Y.Z
   ```

3. **Update files**
   - `Cargo.toml` — bump `version`
   - `Cargo.lock` — run `cargo generate-lockfile`
   - `CHANGELOG.md` — add release notes under new `## [X.Y.Z] - YYYY-MM-DD` section, update comparison links at bottom

4. **Verify**
   ```sh
   mise run verify
   ```

5. **Commit, push, create PR**
   ```sh
   git add Cargo.toml Cargo.lock CHANGELOG.md
   git commit -m "chore: bump version to vX.Y.Z"
   git push -u origin release/vX.Y.Z
   gh pr create -a "@me" -t "chore: bump version to vX.Y.Z (#N)"
   mise run git:open-pr -- <pr#>   # background: CI → browser → merge watch → cleanup
   ```

### Phase 2: Tag (after PR merge)

```sh
mise run release:tag
```

This reads the version from `Cargo.toml` on `origin/main`, creates an annotated tag, and pushes it. GitHub Actions triggers automatically on `v*` tags.

### Phase 3: Watch CI

```sh
mise run release:watch   # run in background
```

Monitors the release workflow triggered by the tag push. Reports:
- Build status (linux, macos, windows)
- GitHub Release creation
- crates.io publish result

## What GitHub Actions Does (on tag push)

1. **Build** — cross-compile for Linux, macOS, Windows + SHA256 checksums
2. **Release** — create GitHub Release with binaries and changelog notes
3. **Publish** — publish to crates.io

## Troubleshooting: crates.io Publish Failure

Common causes:
- **Token issue** — check `CARGO_REGISTRY_TOKEN` secret in repo settings
- **Already published** — version already exists on crates.io
- **Dirty Cargo.lock** — ensure `Cargo.lock` is committed

Recovery options:
```sh
# Option A: re-run just the failed job
gh run rerun <run-id> --failed

# Option B: delete tag, fix, re-tag
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
gh release delete vX.Y.Z --yes
# ... fix via PR ...
mise run release:tag
```
