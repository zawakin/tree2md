# Context CLI: LLM-Friendly Tree Command with Code Blocks

`context-cli` is a command-line tool that scans a given directory and outputs its structure in Markdown format, including both files and directories. It can also display code files (e.g. `.py`, `.go`) as Markdown code blocks, making it easier to review project files at a glance.

## Features

- **File Structure in Markdown:**
  Display directories and files as a Markdown tree.
- **Code Blocks for Supported Files:**
  `.py` and `.go` files are automatically included as code blocks for easy viewing.
- **Modes:**
  - **full** (default): Show all files and directories as a tree, and display code blocks.
  - **tree**: Show all files and directories as a tree only (no code blocks).
- **Language Support:**
  Use `--lang=en` or `--lang=ja` to switch UI text. By default, `en` is used.

## Installation

1. Ensure you have Go installed.
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

- `--all` : Show hidden files and directories as well.
- `--pattern="*.py"` : Filter files by a glob pattern.
- `--mode=full|tree` : Set output mode. Default is `full`.
- `--lang=en|ja` : Set language for UI text.

### Example

Suppose your directory structure looks like this:

```
sample/
  foo/
    bar.go
    bar.py
  hello.py
```

When you run `context-cli ./sample`, the default mode (`full`) will produce something like:


````
```
## File Structure
- .
  - foo
    - bar.go
    - bar.py
  - hello.py

### foo/bar.go
```go
// foo/bar.go
package foo

func Bar() {
}
```

### foo/bar.py
```python
# foo/bar.py
print("foo/bar")
```

### hello.py
```python
# hello.py
print("hello")
```
````

In `tree` mode, you would only see the Markdown tree (no code blocks):

````
## File Structure
- .
  - foo
    - bar.go
    - bar.py
  - hello.py
````

If you run `context-cli --pattern="*.py" ./sample`, only `.py` files will appear in the tree (plus their code blocks if in `full` mode):

````

## File Structure
- .
  - foo
    - bar.py
  - hello.py

### foo/bar.py
```python
# foo/bar.py
print("foo/bar")
```

### hello.py
```python
# hello.py
print("hello")
```
````

These examples help you visualize how `context-cli` formats the directory structure and code files, allowing you to quickly get an overview of a project.

## License

This project is released under the MIT License. See [LICENSE](LICENSE) for details.

## Contributing

Issues and PRs are welcome. Feel free to suggest improvements or new features.
