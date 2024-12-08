# context-cli

`context-cli` is a command-line tool that scans a given directory and outputs its structure in Markdown format, including files and directories. It can also display code files (e.g., `.py`, `.go`) as Markdown code blocks, making it easier to review project files.

## Features

- **File Structure in Markdown:**
  Display directories and files as a Markdown tree.
- **Code Blocks for Supported Files:**
  `.py` and `.go` files are automatically included as code blocks for easy viewing.
- **Modes:**
  - **full** (default): Show all files and directories as a tree, and display code blocks.
  - **tree**: Show all files and directories as a tree only (no code blocks).
- **Language Support:**
  Use `--lang=en` or `--lang=ja` to switch the UI text. By default, `ja` is used.

## Installation

1. Make sure you have Go installed.
2. Clone the repository and build:
   ```bash
   go build -o context-cli .
   ```
3. Place the `context-cli` binary in a directory on your `$PATH` (e.g., `/usr/local/bin`).

## Usage

```bash
context-cli [OPTIONS] <directory>
```

### Options

- `--all`: Show hidden files and directories as well.
- `--pattern="*.py"`: Filter files by a glob pattern.
- `--mode=full|tree`: Set output mode. Default is `full`.
- `--lang=en|ja`: Set language for UI text.

### Examples

- Show full file structure of `./sample` and code blocks:
  ```bash
  context-cli ./sample
  ```

- Show only the tree (no code blocks):
  ```bash
  context-cli --mode=tree ./sample
  ```

- Show only `.py` files in the structure:
  ```bash
  context-cli --pattern="*.py" ./sample
  ```

## License

This project is released under the MIT License. See [LICENSE](LICENSE) for details.

## Contributions

Issues and PRs are welcome.
