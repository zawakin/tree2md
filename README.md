# tree2md

A command-line tool that scans directories and outputs their structure in Markdown format. Can optionally include file contents as code blocks with syntax highlighting.

## Features

- Generate Markdown-formatted directory trees
- Include file contents as syntax-highlighted code blocks
- Filter files by extension
- Find files using wildcard patterns (glob)
- Respect `.gitignore` patterns
- Truncate large files by bytes or lines
- Support for hidden files and directories
- Read file paths from stdin for precise control
- Flat output format for discrete file collections
- Security boundary enforcement with `--restrict-root`
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
- Linux (x86_64)
- macOS (Apple Silicon)
- Windows (x86_64)

## Usage

### Common Use Cases

```bash
# Copy project structure to clipboard for documentation (macOS)
tree2md src -c | pbcopy

# Copy project structure to clipboard (Linux)
tree2md src -c | xclip -selection clipboard

# Copy project structure to clipboard (Windows)
tree2md src -c | clip

# Generate README documentation
tree2md src -c > PROJECT_STRUCTURE.md

# Quick overview without file contents
tree2md src | pbcopy
```

### All Options

```bash
# Basic usage - output tree structure of current directory
tree2md

# Scan specific directory
tree2md /path/to/directory

# Include file contents as code blocks
tree2md -c

# Filter by extensions
tree2md -e .rs,.toml

# Find files matching wildcard patterns
tree2md -f "*.rs"                    # All .rs files
tree2md -f "src/**/*.rs"             # All .rs files under src/
tree2md -f "*.toml" -f "*.md"        # Multiple patterns

# Include hidden files
tree2md -a

# Respect .gitignore
tree2md --respect-gitignore

# Truncate file contents
tree2md -c --max-lines 50
tree2md -c --truncate 1000

# Combine options
tree2md -f "src/**/*.rs" -c --max-lines 100
```

### Stdin Mode

Read file paths from stdin for precise control over which files to include:

```bash
# Document only Git-tracked TypeScript files
git ls-files "*.ts" | tree2md --stdin -c

# Document recently changed files
git diff --name-only HEAD~1 | tree2md --stdin

# Use with find for null-delimited paths (handles spaces/special chars)
find src -type f -name "*.rs" -print0 | tree2md --stdin0

# Expand directories found in stdin
printf '%s\n' src tests | tree2md --stdin --expand-dirs

# Keep input order (useful for prioritized documentation)
echo -e "README.md\nsrc/main.rs\nCargo.toml" | tree2md --stdin --keep-order

# Restrict paths to project directory (security)
rg -l "TODO" | tree2md --stdin --restrict-root "$(pwd)"

# Merge stdin with directory scan
rg --files --type rust | tree2md --stdin --stdin-mode merge src

# Use flat format for discrete file collections
fzf -m | tree2md --stdin --flat -c

# Use relative paths (default in v0.4.0+)
find src -name "*.json" | tree2md --stdin -c

# Show display root for reproducibility
git ls-files | tree2md --stdin --show-root

# Strip common prefix from paths
find ~/projects/myapp -type f | tree2md --stdin --strip-prefix ~/projects
```

## Options

### Basic Options

- `-c, --contents` - Include file contents as code blocks
- `-t, --truncate <N>` - Truncate file content to the first N bytes
- `--max-lines <N>` - Limit file content to the first N lines
- `-e, --include-ext <EXTS>` - Comma-separated list of extensions to include (e.g., .go,.py)
- `-f, --find <PATTERNS>` - Find files matching wildcard patterns (can be used multiple times)
- `-a, --all` - Include hidden files and directories
- `--respect-gitignore` - Respect .gitignore files
- `-h, --help` - Print help information
- `-V, --version` - Print version information

### Stdin Mode Options

- `--stdin` - Read file paths from stdin (newline-delimited)
- `--stdin0` - Read file paths from stdin (null-delimited, for paths with spaces)
- `--stdin-mode <authoritative|merge>` - How to handle stdin input (default: authoritative)
  - `authoritative`: Use only files from stdin
  - `merge`: Combine stdin files with directory scan
- `--keep-order` - Preserve the input order from stdin (default: sort alphabetically)
- `--base <DIR>` - Base directory for resolving relative paths from stdin (default: current directory)
- `--restrict-root <DIR>` - Ensure all paths are within this directory (security feature)
- `--expand-dirs` - Expand directories found in stdin to their contents
- `--flat` - Use flat output format instead of tree structure

### Display Path Options

- `--display-path <relative|absolute|input>` - How to display paths (default: relative)
  - `relative`: Show paths relative to display root
  - `absolute`: Show absolute paths
  - `input`: Show paths as provided in stdin
- `--display-root <DIR>` - Custom root for relative path display (default: auto-detect via LCA)
- `--strip-prefix <PREFIX>` - Remove prefix from display paths (can be used multiple times)
- `--show-root` - Show the display root at the beginning of output
- `--no-root` - Don't show root node in tree output (default for stdin mode)
- `--root-label <LABEL>` - Custom label for root node (e.g., ".", "PROJECT_ROOT")
- `--pure-json` - Keep JSON output pure without comment lines

## Example Output

````markdown
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
````

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
