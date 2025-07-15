# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`tree2md` is a command-line tool written in Go that scans directories and outputs their structure in Markdown format. It can display file structures as trees and optionally include code blocks for supported file types.

## Build and Development Commands

### Building
```bash
go build -o tree2md .
```

### Running
```bash
./tree2md [OPTIONS] <directory>
```

### Testing
```bash
go test .
go test -v .  # verbose output
```

### Installation
```bash
go install github.com/zawakin/tree2md@latest
```

## Architecture

### Core Components

- **Single-file architecture**: All code is contained in `main.go`
- **Tree structure**: Uses a `Node` struct to represent file/directory hierarchy
- **Language detection**: Built-in support for various file extensions with language-specific comment formatting
- **Filtering**: Supports extension-based filtering and hidden file inclusion

### Key Data Structures

- `Node`: Represents files/directories with path, name, type, children, and content
- `Lang`: Defines language detection and comment formatting for code blocks

### Command-line Options

- `-c/--contents`: Include file contents as code blocks
- `-t/--truncate`: Truncate file content to first N bytes
- `--max-lines`: Limit file content to first N lines
- `-e/--include-ext`: Filter by file extensions
- `-a/--all`: Include hidden files and directories
- `-v/--version`: Show version information

### Supported Languages

The tool has built-in language detection for: Go, Python, Shell, JavaScript, TypeScript, TSX, HTML, CSS, SCSS, Sass, and SQL.

## Development Workflow

### Quality-First Feature Development

When implementing new features, follow this systematic workflow to ensure high quality:

#### Phase 1: Initial Implementation
1. **Plan and document** the feature requirements in `z_memo/` directory
2. **Implement core functionality** with basic error handling
3. **Add initial unit tests** for main functionality
4. **Commit initial implementation** for baseline

#### Phase 2: Quality Evaluation
1. **Conduct thorough testing** with real-world edge cases
2. **Document findings** in quality evaluation memo
3. **Identify critical bugs** and quality issues
4. **Prioritize fixes** (Critical → High → Medium → Low)

#### Phase 3: Bug Fixes and Enhancement
1. **Fix critical bugs first** with focused commits
2. **Add comprehensive edge case tests** (empty files, boundary values, etc.)
3. **Update existing tests** to match corrected behavior
4. **Verify all tests pass** before proceeding

#### Phase 4: Release Process
1. **Create feature branch** from main
2. **Push branch and create PR** with detailed description
3. **Merge PR** after review
4. **Pull latest main** and update version number in `main.go` (search for `version = "v`)
5. **Create annotated git tag** with release notes:
   ```bash
   git tag v0.x.y -a -m "Release v0.x.y: Feature description
   
   - Key feature 1
   - Key feature 2
   - Bug fixes and improvements"
   ```
6. **Push tag** for release: `git push origin v0.x.y`

### Quality Standards

- **Code Quality Target**: A- (85/100+)
- **Test Coverage**: Include edge cases, boundary values, and error conditions
- **Bug Tolerance**: Zero critical bugs in production
- **Documentation**: Update CLAUDE.md and relevant docs

### Common Edge Cases to Test

- Empty files (0 bytes)
- Files without trailing newlines
- Single-line files
- Very large files
- Boundary values (negative, zero, minimum values)
- Error conditions (file permissions, I/O errors)

## Development Notes

- No external dependencies beyond Go standard library
- Version is hardcoded in `main.go` (search for `version = "v`)
- Language support can be extended by modifying the `langs` slice (search for `var langs = []Lang`)
- Filter logic operates on the tree structure after initial building