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

## Development Notes

- No external dependencies beyond Go standard library
- Version is hardcoded in `main.go:14`
- Language support can be extended by modifying the `langs` slice in `main.go:264-276`
- Filter logic operates on the tree structure after initial building