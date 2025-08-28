# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2025-08-28
### Added
- Stdin input mode with `--stdin` and `--stdin0` options
  - Read file paths from stdin for precise control
  - Support for null-delimited input (`--stdin0`) for paths with spaces
  - `--stdin-mode` option for authoritative or merge modes
  - `--keep-order` to preserve input order from stdin
  - `--base` for resolving relative paths
  - `--restrict-root` for security boundary enforcement
  - `--expand-dirs` to expand directories found in stdin
  - `--flat` output format for discrete file collections
- New `stdin` module for input processing
- Integration tests for stdin functionality

### Changed
- Default `.gitignore` respect behavior differs in stdin mode (off by default in authoritative mode)
- Enhanced release process with Claude-driven automation

### Fixed
- Markdown code block escaping in README examples

### Notes
- No breaking changes - all existing functionality preserved
- Stdin mode is completely optional and backward compatible

## [0.3.2] - 2025-01-27
### Changed
- Major code refactoring for better maintainability
  - Extract language definitions to separate module (`src/language.rs`)
  - Extract utility functions to separate module (`src/utils.rs`)
  - Reduce main.rs from 729 to 516 lines (~29% reduction)
- Improved code organization and modularity
- Better test structure with module-specific tests

### Fixed
- Add `allow(dead_code)` for `format_size` function (will be used in future features)

## [0.3.1] - 2025-01-25
### Changed
- Sort directories before files in output for better readability
- Show warnings for skipped non-UTF8 paths instead of failing silently
- Simplify release process to single-command automation
- Integrate crates.io publishing into the main release workflow

### Fixed
- Improve error handling for non-UTF8 file paths
- Normalize path separators on Windows for pattern matching
- Handle `canonicalize` failures for symlinks and special files
- Add fallback when root path canonicalization fails

## [0.3.0] - 2025-01-25
### Added
- Wildcard pattern support with `-f/--find`
- Glob patterns (e.g., `*.rs`, `src/**/*.go`)
- Multiple pattern support（`-f`を複数回指定可能）
- Add `glob` crate dependency for pattern matching

### Changed
- Update README with wildcard examples

## [0.2.0] - 2025-01-17 — Rust Version
### Added
- Complete rewrite in Rust
- Cross-platform binary distribution
- GitHub Actions CI/CD
- crates.io support

### Changed
- Migrate from Go implementation to Rust
- Faster execution
- More robust `.gitignore` handling (using `ignore` crate)

### Maintained
- CLI options compatibility with Go version
- Same output format

---

### Legacy (Go)
> The following entries document the legacy Go implementation history.

## [0.1.6] - 2025-01-22
### Added
- `--respect-gitignore` flag
- File exclusion based on `.gitignore`
- Support for directories, wildcards, and negation patterns

## [0.1.5] - 2025-01-15
### Added
- `--max-lines` option and line limit for file contents
- Display detailed truncation info

### Fixed
- Update version string

## [0.1.4] - 2024-12-31
### Added
- Version information display (`-v`, `--version`)

## [0.1.3] - 2024-12-31
### Added
- HTML language support

## [0.1.2] - 2024-12-08
### Changed
- Change default mode

## [0.1.1] - 2024-12-08
### Added
- MIT License

## [0.1.0] - 2024-12-08
### Added
- Initial release: Markdown directory tree
- Code block display
- Extension filtering
- Hidden file support
- Multi-language support (English/Japanese)

[Unreleased]: https://github.com/zawakin/tree2md/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/zawakin/tree2md/compare/v0.3.2...v0.4.0
[0.3.2]: https://github.com/zawakin/tree2md/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/zawakin/tree2md/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/zawakin/tree2md/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/zawakin/tree2md/compare/v0.1.6...v0.2.0
[0.1.6]: https://github.com/zawakin/tree2md/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/zawakin/tree2md/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/zawakin/tree2md/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/zawakin/tree2md/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/zawakin/tree2md/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/zawakin/tree2md/compare/v0.1.0...v0.1.1
