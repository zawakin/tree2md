# tree2md (Rust Version)

A command-line tool that scans directories and outputs their structure in Markdown format. Can optionally include file contents as code blocks with syntax highlighting.

## Features

- Generate Markdown-formatted directory trees
- Include file contents as syntax-highlighted code blocks
- Filter files by extension
- Respect `.gitignore` patterns
- Truncate large files by bytes or lines
- Support for hidden files and directories
- Fast and efficient, written in Rust

## Installation

### From crates.io

```bash
cargo install tree2md
```

### From source

```bash
git clone https://github.com/zawakin/tree2md.git
cd tree2md
cargo build --release
# Binary will be at ./target/release/tree2md
```

### Pre-built binaries

Download pre-built binaries from the [releases page](https://github.com/zawakin/tree2md/releases).

Available for:
- Linux (x86_64, aarch64, musl)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)

## Usage

```bash
# Basic usage - output tree structure of current directory
tree2md

# Scan specific directory
tree2md /path/to/directory

# Include file contents as code blocks
tree2md -c

# Filter by extensions
tree2md -e .rs,.toml

# Include hidden files
tree2md -a

# Respect .gitignore
tree2md --respect-gitignore

# Truncate file contents
tree2md -c --max-lines 50
tree2md -c --truncate 1000
```

## Options

- `-c, --contents` - Include file contents as code blocks
- `-t, --truncate <N>` - Truncate file content to the first N bytes
- `--max-lines <N>` - Limit file content to the first N lines
- `-e, --include-ext <EXTS>` - Comma-separated list of extensions to include (e.g., .go,.py)
- `-a, --all` - Include hidden files and directories
- `--respect-gitignore` - Respect .gitignore files
- `-h, --help` - Print help information
- `-V, --version` - Print version information

## Example Output

```markdown
## File Structure
- my_project/
  - src/
    - main.rs
    - lib.rs
  - Cargo.toml
  - README.md

### src/main.rs
```rust
// src/main.rs
fn main() {
    println!("Hello, world!");
}
```

### src/lib.rs
```rust
// src/lib.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```
```

## Supported Languages

The tool automatically detects and applies syntax highlighting for:

- Rust (.rs)
- Go (.go)
- Python (.py)
- JavaScript (.js)
- TypeScript (.ts, .tsx)
- HTML (.html)
- CSS (.css, .scss, .sass)
- SQL (.sql)
- Shell scripts (.sh)
- TOML (.toml)
- YAML (.yaml, .yml)
- JSON (.json)
- Markdown (.md)

## Differences from Go Version

This Rust implementation maintains full compatibility with the original Go version while adding:
- Better performance through Rust's zero-cost abstractions
- More robust gitignore pattern matching using the `ignore` crate
- Cross-platform binary distribution
- Available via cargo for easy installation

## Building from Source

Requirements:
- Rust 1.70 or later
- Cargo

```bash
# Clone the repository
git clone https://github.com/zawakin/tree2md.git
cd tree2md

# Build release version
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Original Go Version

The original Go version is available at [github.com/zawakin/tree2md](https://github.com/zawakin/tree2md)