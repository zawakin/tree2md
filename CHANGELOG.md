# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.4] - 2026-02-26

### Fixed
- Prune nested git repositories, worktrees, and submodules — directories containing a `.git` entry are now automatically excluded as separate repository boundaries
- Support `.git/info/exclude` patterns for per-repo local excludes

### Improved
- Make `--max-chars` explicit in help and README

## [0.9.3] - 2026-02-25

### Fixed
- Normalize trailing slash in `-I`/`-X` patterns — `hoge/` and `hoge` now behave identically
- Gitignore/safety now always prune directories before generic include patterns, matching `rg`/`fd` semantics — `-I src` no longer traverses into gitignored directories like `target/`

## [0.9.2] - 2026-02-25

### Fixed
- Support nested `.gitignore` files in subdirectories — previously only root-level `.gitignore` was respected (#22)
- Auto-detection now walks up parent directories to find `.git`, so `tree2md src/` inside a repo correctly respects `.gitignore`
- Check `~/.config/git/ignore` (Git 2.20+ default) before legacy `~/.gitignore` for global gitignore

## [0.9.1] - 2026-02-25

### Fixed
- Correct `-I`/`-X` priority and wildcard pattern bugs (#19)

### Changed
- Migrate release process from command to skill (#20)

## [0.9.0] - 2026-02-25

### Breaking Changes
- **Removed HTML and Markdown renderers** — Removed `src/render/html.rs`, `src/render/markdown.rs`, `src/output/html_tree.rs`, `src/output/links.rs`
- **Removed injection and stamp modules** — `src/injection/`, `src/stamp/`
- **Removed CLI options**: `--output`, `--preset`, `--fold`, `--links`, `--github`, `--inject`, `--tag-start`, `--tag-end`, `--dry-run`, `--restrict-root`, `--stamp*`, `--no-stats`, and related enums

### Added
- **Agent-optimized output modes** with auto-detection:
  - TTY: pretty output with emoji, LOC bars, stats, tree characters
  - Pipe: plain tree + line counts — ideal for `pbcopy`/LLM pipes
  - Pipe + `-c`: tree + code-fenced file contents for full AI context
- **`--max-chars` content truncation** — limits total content characters when using `-c`, with proportional budget allocation across files
  - `head` mode (default): keeps first N chars at line boundaries
  - `nest` mode: progressively collapses deeply-indented blocks
- `src/render/pipe.rs` — PipeRenderer with tree chars and optional contents
- SIGPIPE handling on Unix to avoid panics when piping to `head`/`less`

### Changed
- Pivoted from multi-format document tool to agent-optimized codebase viewer

## [0.8.1] - 2025-09-03
### Documentation
- Enhanced README with improved above-the-fold section
- Added compelling one-liner examples showcasing terminal and GitHub modes
- Added screenshot for immediate visual context
- Updated tagline to highlight multiple output formats
- Improved key features to reflect actual capabilities
- Reorganized Quick Start examples with terminal-first approach
- Added CONTRIBUTING.md with comprehensive development guidelines

## [0.8.0] - 2025-09-03
### Added
- **Major Architecture Overhaul**: Complete refactoring with modular architecture
- **Multiple Output Formats**: 
  - HTML tree visualization with interactive features
  - Terminal output with progress animations and colors
  - Enhanced Markdown rendering with multiple styles
- **New Modules**:
  - `injection`: Smart README injection and content updating
  - `stamp`: Provenance tracking with version, date, and commit info
  - `profile`: Language-specific file type detection with emoji support
  - `output`: Statistics, link generation, and HTML tree creation
  - `render`: Flexible rendering pipeline for multiple output formats
  - `terminal`: Advanced terminal detection and animation capabilities
  - `safety`: Preset configurations and content validation
- **Enhanced CLI Options**:
  - New presets for common use cases (e.g., `--preset rust`, `--preset python`)
  - `--stamp` option for adding metadata (version/date/commit)
  - `--output-format` for choosing between markdown, html, and terminal
  - `--emoji` support for file type indicators
  - `--stats` for displaying file statistics
  - `--links` for generating file links
- **Comprehensive Test Suite**: 
  - Complete test overhaul with modular integration tests
  - Added tests for emoji, filtering, HTML output, links, presets, safety, and more
  - Improved test fixtures and helper utilities

### Changed
- **Refactored Core Modules**: 
  - Restructured `fs_tree` module with better separation of concerns
  - Improved `matcher` engine with enhanced performance
  - Modularized content reading and I/O operations
- **Enhanced Performance**:
  - Progress tracking for large directory traversals
  - Optimized file reading with better memory management
  - Improved glob pattern matching efficiency
- **Better Error Handling**: More informative error messages and validation

### Fixed
- Improved path handling and normalization
- Enhanced glob pattern matching accuracy
- Better handling of edge cases in file traversal

### Developer Experience
- Added `scripts/update-readme-embeds.sh` for README maintenance
- Improved code organization and modularity
- Better separation of concerns across modules
- Enhanced documentation and code comments

## [0.7.0] - 2025-08-29
### Changed
- **BREAKING**: Changed default behavior for hidden files
  - Hidden files and directories are now shown by default
  - Removed `--all` flag (no longer needed)
  - Added `--exclude-hidden` flag to hide dotfiles when needed
  - This aligns with the principle that `.gitignore` is authoritative for exclusions
- **BREAKING**: `.gitignore` is now respected by default
  - Previously required explicit opt-in
  - Use `--no-gitignore` to disable gitignore processing
- **BREAKING**: Removed multiple CLI options for simplification
  - Removed `--stdin0`, `--stdin-mode`, `--keep-order` (stdin simplification)
  - Removed `--base`, `--display-root`, `--show-root`, `--no-root` (display simplification)  
  - Removed `--respect-gitignore` (now default behavior)
- Simple glob patterns (e.g., `*.rs`) are now recursive by default
  - Matches common user expectations
  - `*.rs` now finds all Rust files in the tree, not just current directory

### Added
- `--exclude-hidden` flag to exclude hidden files and directories
- `.git/` directory is now always excluded for safety and cleanliness
- Comprehensive integration test suite

### Fixed
- Correct glob pattern matching behavior for single asterisk
- Stdin mode base directory resolution and gitignore handling
- Gitignore handling in stdin expand mode
- `--flat` option now works correctly

### Improved
- Refactored to MatcherEngine architecture for better performance
- Restructured codebase into modular architecture
- Optimized path matching and filtering
- Delegated hidden file filtering to WalkBuilder for efficiency
- Simplified CLI interface by removing redundant options
- Better code quality and consistency throughout

## [0.6.0] - 2025-08-28
### Added
- Comprehensive gitignore support using `ignore::WalkBuilder`
  - Support for nested `.gitignore` files in subdirectories
  - Support for `.git/info/exclude` patterns
  - Support for global gitignore configuration
  - Support for `.ignore` files
  - Improved performance with efficient pattern matching
- Binary file detection and handling
  - Automatically detect binary files (NULL bytes and control characters)
  - Display "Binary file (size)" with human-readable sizes (B, KB, MB, GB)
  - Prevent garbled output from binary files

### Changed
- Directory names now display with trailing `/` for clearer distinction
- Improved Cargo.toml description to match README tagline
- Enhanced gitignore processing with better caching and performance

### Fixed
- Critical stdin double-read issue when using `--display-path input`
  - Introduced `StdinResult` struct to capture both canonical paths and original inputs
  - Process stdin only once and maintain proper mapping
- Multi-byte character handling in byte truncation
  - Use `char_indices()` to find safe UTF-8 boundaries
  - Prevent character corruption at truncation points
- JSON truncation annotations
  - Print truncation message outside code block for JSON files
  - JSON doesn't support comments, avoiding syntax errors
- GitHub Actions release workflow to include CHANGELOG in release notes

## [0.5.0] - 2025-08-28
### Changed
- **BREAKING**: Removed file name comments from code blocks
  - File names are no longer duplicated as comments inside code blocks
  - Markdown headers already provide file identification
  - Improves copy-paste usability and reduces token count for LLM usage
- **BREAKING**: Removed `--pure-json` option
  - JSON files now always use standard `json` language tag
  - Simplifies output format and removes unnecessary complexity
- Enhanced README documentation
  - Added feature comparison table vs `tree` command
  - Improved structure with table of contents
  - Added Quick Start section with clipboard examples
  - Added crates.io and license badges

### Notes
- This release contains breaking changes to the output format
- The cleaner output aligns with industry standards (GitHub, GitLab)
- Tests added to ensure proper output formatting

## [0.4.1] - 2025-08-28
### Added
- Display path control options for improved output flexibility
  - `--display-path` option: Choose between relative (default), absolute, or input paths
  - `--display-root` option: Specify custom root for relative path calculation (default: auto-detect via LCA)
  - `--strip-prefix` option: Remove specified prefix from display paths
  - `--show-root` flag: Show the display root at the beginning of output
- Root node display control
  - `--no-root` flag: Don't show root node in tree (default for stdin mode)
  - `--root-label <LABEL>` option: Custom label for root node (e.g., ".", "PROJECT_ROOT")
  - Stdin mode now defaults to no root display for better portability
- JSON output improvements
  - Automatic `jsonc` language detection for JSON files with comments
  - `--pure-json` flag to keep JSON output pure (no comments)
- Better path handling for stdin mode
  - Relative paths by default instead of absolute paths
  - LCA (Lowest Common Ancestor) auto-detection for display root
  - Original input preservation for `--display-path input` mode

### Fixed
- Absolute paths no longer shown in headers and comments by default
- Improved privacy by not exposing system paths in output
- Stdin mode no longer exposes CWD name by default

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

[Unreleased]: https://github.com/zawakin/tree2md/compare/v0.9.4...HEAD
[0.9.4]: https://github.com/zawakin/tree2md/compare/v0.9.3...v0.9.4
[0.9.3]: https://github.com/zawakin/tree2md/compare/v0.9.2...v0.9.3
[0.9.2]: https://github.com/zawakin/tree2md/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/zawakin/tree2md/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/zawakin/tree2md/compare/v0.8.2...v0.9.0
[0.8.2]: https://github.com/zawakin/tree2md/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/zawakin/tree2md/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/zawakin/tree2md/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/zawakin/tree2md/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/zawakin/tree2md/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/zawakin/tree2md/compare/v0.4.1...v0.5.0
[0.4.1]: https://github.com/zawakin/tree2md/compare/v0.4.0...v0.4.1
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
