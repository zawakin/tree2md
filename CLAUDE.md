# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

`tree2md` is a command-line tool written in **Rust** that scans directories and outputs their structure in Markdown format with optional code blocks.

## Quick Commands

For common development tasks, use the following commands:

### Development
- **Build**: `cargo build --release`
- **Test**: `cargo test`
- **Run**: `./target/release/tree2md [OPTIONS] <directory>`
- **Format**: `cargo fmt`
- **Lint**: `cargo clippy -- -D warnings`

### Release Process (Simplified)
1. **Update version**: Manually edit `Cargo.toml`, `src/main.rs`, and `CHANGELOG.md`
2. **Release**: `./scripts/release.sh vX.Y.Z`
3. **CI automatically**: Builds binaries, creates GitHub Release, publishes to crates.io

### Documentation
- `release.md` - Simple release steps

## Important Notes

- Written in Rust (not Go anymore)
- Version in `Cargo.toml` and `src/main.rs` must match
- GitHub Actions handles CI/CD and releases
- crates.io publishing is automated on tag push