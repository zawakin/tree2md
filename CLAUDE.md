# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

`tree2md` is a command-line tool written in **Rust** that scans directories and outputs their structure in Markdown format with optional code blocks.

## Quick Commands

For common development tasks, use the following commands:

### 📋 Development
- **Build**: `cargo build --release`
- **Test**: `cargo test`
- **Run**: `./target/release/tree2md [OPTIONS] <directory>`

### 🚀 Release Process
- **Pre-release test**: `./scripts/test-release.sh`
- **Automated release**: `./scripts/release.sh [patch|minor|major]`
- **Manual release checklist**: See `.claude/commands/pre-release-checklist.md`

### 📚 Documentation
All detailed documentation and workflows are in `.claude/commands/`:

- `release.md` - Manual release steps
- `pre-release-checklist.md` - Complete release checklist

## Important Notes

- Written in Rust (not Go anymore)
- Version in `Cargo.toml` and `src/main.rs` must match
- GitHub Actions handles CI/CD and releases
- crates.io publishing is automated on tag push