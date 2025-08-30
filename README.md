
# tree2md

[![Crates.io](https://img.shields.io/crates/v/tree2md.svg)](https://crates.io/crates/tree2md)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Like the `tree` command, but outputs in Markdown.**
Scans directories and prints a Markdown-formatted tree. Optionally embed file contents as syntax-highlighted code blocks.

> [!NOTE]
> This project is still in an early stage and changing rapidly 🚧  
> Contributions are welcome, but responses may take time until things stabilize.  
> Beginner-friendly issues will be added once the codebase is more settled.

---

## Table of Contents
- [tree2md](#tree2md)
  - [Table of Contents](#table-of-contents)
  - [Quick Start](#quick-start)
  - [Why tree2md vs. `tree`](#why-tree2md-vs-tree)
  - [Features](#features)
  - [Installation](#installation)
    - [From crates.io](#from-cratesio)
    - [From source](#from-source)
    - [Pre-built binaries](#pre-built-binaries)
  - [Usage](#usage)
    - [Common recipes](#common-recipes)
      - [Git-friendly](#git-friendly)
    - [All options (cheat sheet)](#all-options-cheat-sheet)
  - [Stdin mode (precise control)](#stdin-mode-precise-control)
  - [Display \& path controls](#display--path-controls)
  - [Example output](#example-output)
  - [Supported languages](#supported-languages)
  - [Performance \& security](#performance--security)
  - [Build from source](#build-from-source)
  - [Contributing](#contributing)
  - [License](#license)

---

## Quick Start

```bash
# Install
cargo install tree2md

# Show directory tree (no file contents)
tree2md src > PROJECT_STRUCTURE.md

# Include file contents as code blocks
tree2md src -c > PROJECT_STRUCTURE.md
```

Clipboard helpers:

```bash
# macOS
tree2md src -c | pbcopy
# Linux
tree2md src -c | xclip -selection clipboard
# Windows
tree2md src -c | clip
```

---

## Why tree2md vs. `tree`

| Capability                            | `tree` | `tree2md` |
| ------------------------------------- | :----: | :-------: |
| Output Markdown                       |   ✖︎   |     ✔     |
| Embed file contents (code blocks)     |   ✖︎   |     ✔     |
| Syntax highlighting hints             |   ✖︎   |     ✔     |
| Respect `.gitignore`                  |   ◯\*  |     ✔     |
| Filter by extension / glob            |    ◯   |     ✔     |
| Drive via stdin (authoritative/merge) |    △   |     ✔     |
| Flat output for file collections      |   ✖︎   |     ✔     |
| Truncate by bytes / lines             |   ✖︎   |     ✔     |
| Security boundary (`--restrict-root`) |   ✖︎   |     ✔     |
| Fast, single-binary (Rust)            |    —   |     ✔     |

\* depending on platform/flags; `tree2md` respects `.gitignore` by default.

---

## Features

* Markdown-formatted directory trees
* Optional file contents as fenced code blocks (with language hints)
* Extension filters and glob patterns
* Honors `.gitignore` by default (use `--no-gitignore` to disable)
* Truncate large files by **bytes** or **lines**
* Hidden files/dirs shown by default (use `--exclude-hidden` to hide)
* `.git/` directory is always excluded for safety and cleanliness
* Read paths from **stdin** (newline or NUL-delimited)
* **Flat** output for discrete file sets
* Security guardrail with `--restrict-root`
* Fast & efficient (Rust)

---

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
# Binary at: ./target/release/tree2md
```

### Pre-built binaries

Download from the [releases page](https://github.com/zawakin/tree2md/releases).

Available for:

* Linux (x86\_64)
* macOS (Apple Silicon)
* Windows (x86\_64)

---

## Usage

### Common recipes

```bash
# Quick overview without contents
tree2md .

# Generate README-style docs with contents
tree2md src -c > PROJECT_STRUCTURE.md

# Filter by extensions
tree2md src -e .rs,.toml

# Find with glob patterns (repeatable)
tree2md -f "*.rs" -f "src/**/*.rs"

# Exclude hidden files (shown by default)
tree2md --exclude-hidden

# Truncate embedded contents (lines or bytes)
tree2md -c --max-lines 80
tree2md -c --truncate 2000

# 7) Combine filters + contents + truncation
tree2md -f "src/**/*.rs" -c --max-lines 100
```

#### Git-friendly

```bash
# Only Git-tracked TypeScript files
git ls-files "*.ts" | tree2md --stdin -c

# Recently changed files
git diff --name-only HEAD~1 | tree2md --stdin
```

### All options (cheat sheet)

**Basic**

* `-c, --contents` — include file contents as code blocks
* `-t, --truncate <N>` — truncate file content to first **N bytes**
* `--max-lines <N>` — limit file content to first **N lines**
* `-e, --include-ext <EXTS>` — comma-separated list (e.g. `.go,.py`)
* `-f, --find <PATTERN>` — glob pattern (repeatable), e.g. `"src/**/*.rs"`
* `--exclude-hidden` — exclude hidden files/dirs (dotfiles)
* `--no-gitignore` — ignore `.gitignore` files and include all files
* `-h, --help` / `-V, --version`

**Note:** The `.git/` directory is always excluded regardless of flags.

**Stdin mode**

* `--stdin` — read newline-delimited paths from stdin
* `--restrict-root <DIR>` — ensure all paths stay within this directory
* `--expand-dirs` — expand directories received via stdin
* `--flat` — render a flat list (no tree)

**Display & paths**

* `--display-path <relative|absolute|input>` — default: **relative**
* `--strip-prefix <PREFIX>` — remove prefix from display paths (repeatable)
* `--root-label <LABEL>` — custom label for the root (e.g. `"."`, `"PROJECT_ROOT"`)

---

## Stdin mode (precise control)

Use stdin when you already have an exact file list (CI pipelines, `git ls-files`, `ripgrep`, `find`).

```bash
# Only use files from stdin (no directory scanning)
git ls-files "*.ts" | tree2md --stdin -c

# Expand directories that appear in stdin
printf '%s\n' src tests | tree2md --stdin --expand-dirs

# Enforce a project boundary (security)
rg -l "TODO" | tree2md --stdin --restrict-root "$(pwd)"
```

**Flat mode** is great for discrete file collections:

```bash
fzf -m | tree2md --stdin --flat -c
```

---

## Display & path controls

```bash
# Show absolute paths
tree2md --display-path absolute .

# Strip a common prefix from printed paths
find ~/projects/myapp -type f | tree2md --stdin --strip-prefix ~/projects
```

---

## Example output

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
fn main() {
    println!("Hello, world!");
}
```

### src/lib.rs
```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```
````

---

## Supported languages

Language detection for fenced code blocks (non-exhaustive):

* Rust (.rs), Go (.go), Python (.py)
* JavaScript (.js), TypeScript (.ts, .tsx)
* HTML (.html), CSS (.css, .scss, .sass)
* SQL (.sql), Shell (.sh)
* TOML (.toml), YAML (.yaml, .yml)
* JSON (.json), Markdown (.md)

---

## Performance & security

* Designed to be fast and memory-efficient.
* Use `--truncate` / `--max-lines` for very large files.
* Prefer `--respect-gitignore` to avoid noise.
* Use `--restrict-root` in scripts/CI to prevent path escapes.

---

## Build from source

Requirements:

* Rust **1.70** or later
* Cargo

```bash
git clone https://github.com/zawakin/tree2md.git
cd tree2md

# Build release
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

---

## Contributing

Issues and PRs are welcome. Please include a clear description and, if possible, tests.

---

## License

MIT License
