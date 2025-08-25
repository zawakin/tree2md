# Changelog

## [0.3.1] - 2025-01-25

### Fixed
- Improved error handling for non-UTF8 file paths
- Fixed pattern matching on Windows (normalize path separators)
- Fixed canonicalize failures for symlinks and special files
- Better fallback when root path canonicalization fails

### Improved
- Sort directories before files in output for better readability
- Added warnings for skipped non-UTF8 paths instead of silent failures
- Simplified release process to single-command automation
- Integrated crates.io publishing into main release workflow

## [0.3.0] - 2025-01-25

### Added
- Wildcard pattern support with `-f/--find` option
- Support for glob patterns (e.g., `*.rs`, `src/**/*.go`)
- Multiple pattern support (can use `-f` multiple times)
- `glob` crate dependency for pattern matching

### Improved
- README documentation updated with wildcard examples

## [0.2.0] - 2025-01-17 - Rust Version

### Added
- Complete rewrite in Rust
- Cross-platform binary distribution
- GitHub Actions CI/CD
- crates.io support

### Changed
- Migrated from Go implementation to Rust
- Faster execution
- More robust .gitignore handling (using ignore crate)

### Maintained
- All command-line options compatibility
- Same output format

## [0.1.6] - 2025-01-22 (Go Version)

### Added
- `--respect-gitignore` flag
- File exclusion based on .gitignore patterns
- Support for directories, wildcards, and negation patterns

## [0.1.5] - 2025-01-15 (Go Version)

### Added
- `--max-lines` option
- Line limit feature for file contents
- Detailed truncation information display

### Fixed
- Version string update

## [0.1.4] - 2024-12-31 (Go Version)

### Added
- Version information display (`-v`, `--version`)

## [0.1.3] - 2024-12-31 (Go Version)

### Added
- HTML language support

## [0.1.2] - 2024-12-08 (Go Version)

### Changed
- Changed default mode

## [0.1.1] - 2024-12-08 (Go Version)

### Added
- MIT License

## [0.1.0] - 2024-12-08 (Go Version)

### Initial Release
- Markdown output of directory structure
- Code block display feature
- Extension filtering
- Hidden file support
- Multi-language support (English/Japanese)