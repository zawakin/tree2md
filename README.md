# tree2md

[![Crates.io](https://img.shields.io/crates/v/tree2md.svg)](https://crates.io/crates/tree2md)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Visualize your codebase structure for humans and AI agents.**

```bash
tree2md                        # Pretty tree in terminal
tree2md | pbcopy               # Pipe-friendly tree for clipboard
tree2md -c -I "*.rs" -L 2     # Tree + file contents for AI context
tree2md -c --max-chars 30000   # Fit contents within token budget
```

---

## Output Modes

Output format is auto-detected based on TTY:

| Mode | When | What |
|------|------|------|
| **TTY** | Terminal | Emoji, LOC bars, stats, tree characters |
| **Pipe** | `\| pbcopy`, redirect, etc. | Plain tree + `(N lines)` |
| **Pipe + `-c`** | AI context | Tree + code-fenced file contents |

### TTY (terminal)

```
├── 📁 src/
│   ├── 🦀 cli.rs          [██████████]    156 (M)
│   ├── 🦀 main.rs         [████······]     65 (S)
│   └── 📁 render/
│       ├── 🦀 pipe.rs     [█████████·]    148 (M)
│       └── 🦀 terminal.rs [██████████]    247 (M) ★
└── Cargo.toml              [█·········]     36 (S)

**Totals**: 📂 3 dirs • 📄 5 files • 🧾 ~652 LOC
```

### Pipe

```
.
├── src/
│   ├── cli.rs  (156 lines)
│   ├── main.rs  (65 lines)
│   └── render/
│       ├── pipe.rs  (148 lines)
│       └── terminal.rs  (247 lines)
└── Cargo.toml  (36 lines)
```

### Pipe + `-c` (AI context)

```
.
├── src/
│   └── main.rs  (65 lines)
└── Cargo.toml  (36 lines)

## src/main.rs

```rust
fn main() {
    println!("Hello, world!");
}
```

## Cargo.toml

```toml
[package]
name = "example"
```
```

---

## Installation

```bash
cargo install tree2md
```

Or from source:

```bash
git clone https://github.com/zawakin/tree2md
cd tree2md
cargo install --path .
```

---

## CLI Options

### Filtering

| Flag | Description |
|------|-------------|
| `-L, --level <N>` | Limit traversal depth |
| `-I, --include <GLOB>` | Include patterns (repeatable) |
| `-X, --exclude <GLOB>` | Exclude patterns (repeatable) |
| `--use-gitignore {auto\|never\|always}` | Respect `.gitignore` |
| `-l, --follow-links` | Follow symbolic links (default: symlinks are skipped) |
| `--paths-from <FILE>` | Include only the paths listed in FILE, one per line (`-` = stdin). Literal paths, no globs; a listed directory includes its contents |
| `--files0-from <FILE>` | Same as `--paths-from`, NUL-separated (`find -print0`, `git ls-files -z`, `fd -0`) |

`-I`/`-X` can be combined with a path list to refine it — rules are applied on top with the usual last-match-wins order (`--paths-from list -X build` prunes `build/` inside listed directories).

### Contents

| Flag | Description |
|------|-------------|
| `-c, --contents` | Append file contents as code blocks |
| `--max-chars <N>` | Limit total content to N characters (requires `-c`) |
| `--contents-mode {head\|nest}` | Truncation strategy (default: `head`) |

### Statistics

| Flag | Description |
|------|-------------|
| `--stats {off\|min\|full}` | Statistics display (default: `full`) |
| `--loc {off\|fast\|accurate}` | Line counting mode (default: `fast`) |

### Fun & Style

| Flag | Description |
|------|-------------|
| `--fun {auto\|on\|off}` | Emojis and animations (default: `auto`) |
| `--emoji <MAPPING>` | Custom emoji (e.g., `--emoji ".rs=🚀"`) |
| `--emoji-map <FILE>` | Load emoji mappings from TOML file |
| `--no-anim` | Disable animations |

### Safety

| Flag | Description |
|------|-------------|
| `--safe` | Apply safety filters (default) |
| `--unsafe` | Disable all safety filters |

---

## Safety Defaults

Excluded by default:

- `.env`, `.ssh/**`, `*.pem`, `*.key`
- `node_modules/`, `target/`, `dist/`, `build/`
- `.git/**`, `.DS_Store`, `Thumbs.db`

Use `-I` to selectively include, or `--unsafe` to disable filters.

---

## Use Cases

**Copy structure to clipboard**

```bash
tree2md . -L 3 | pbcopy
```

**Feed codebase to AI agent**

```bash
tree2md . -c -I "*.rs" -I "*.toml" | pbcopy
```

**Fit contents within token budget**

```bash
tree2md . -c --max-chars 30000 | pbcopy
```

**Quick project overview**

```bash
tree2md . -L 2 --stats min
```

**Rust files only, 3 levels deep**

```bash
tree2md src/ -L 3 -I "*.rs"
```

**Select directories by a condition tree2md can't express** — compute the
path set with your favourite tool and pipe it in:

```bash
# only projects whose project.json says "active"
for d in projects/*/; do
  jq -e '.status == "active"' "$d/project.json" >/dev/null && printf '%s\0' "$d"
done | tree2md -c --files0-from -

# tracked files only
git ls-files -z | tree2md --files0-from -

# files changed on this branch, with contents, for a review prompt
git diff --name-only main... | tree2md -c --paths-from - | pbcopy
```

---

## Build from Source

```bash
git clone https://github.com/zawakin/tree2md
cd tree2md
cargo build --release
cargo test
```

---

## Contributing

PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

```bash
mise run verify   # fmt + clippy + tests
mise run fix      # auto-format + clippy fix
```

---

## License

MIT License — see [LICENSE](LICENSE).
