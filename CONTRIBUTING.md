# Contributing to tree2md

Thank you for your interest in contributing to tree2md! We welcome contributions from the community and appreciate your help in making this tool better.

## Getting Started

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git
- GitHub account

### Setting Up Development Environment

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/<your-username>/tree2md
   cd tree2md
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/zawakin/tree2md
   ```
4. Build the project:
   ```bash
   cargo build
   ```
5. Run tests to verify setup:
   ```bash
   cargo test
   ```

## Development Workflow

### Building

```bash
cargo build          # Debug build
cargo build --release  # Release build
```

### Testing

```bash
cargo test                # Run all tests
cargo test --test '*'     # Run integration tests
cargo test <test_name>    # Run specific test
```

### Code Quality

Before submitting a PR, ensure your code passes all checks:

```bash
cargo fmt                    # Format code
cargo clippy -- -D warnings  # Run linter
cargo test                   # Run all tests
```

### Debugging

For debugging, use integration tests in `tests/` instead of using `/tmp` directory in bash commands. The integration test framework provides proper test fixtures and cleanup.

## Project Structure

```
tree2md/
├── src/
│   ├── main.rs       # Entry point
│   └── cli.rs        # CLI argument parsing and core logic
├── tests/            # Integration tests
│   ├── fixtures.rs   # Test utilities and fixtures
│   └── *_test.rs     # Test files
└── Cargo.toml        # Project configuration
```

## Contributing Guidelines

### Issues

- **Bug Reports**: Include reproduction steps, expected vs actual behavior, and environment details
- **Feature Requests**: Describe the use case and proposed solution
- **Questions**: Check existing issues and documentation first

### Pull Requests

1. **Create an issue first** for significant changes to discuss the approach
2. **One feature per PR** - Keep PRs focused and reviewable
3. **Update tests** - Add tests for new features and bug fixes
4. **Update documentation** - Keep README.md and inline docs current
5. **Follow existing patterns** - Match the codebase style and conventions

### Commit Messages

Follow conventional commit format:

```
type: subject

[optional body]

[optional footer(s)]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions/changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

Example:
```
feat: add support for custom markdown tags

Allows users to specify custom start/end tags for README injection
using --tag-start and --tag-end flags.

Closes #42
```

### Code Style

- Follow Rust conventions and idioms
- Use `cargo fmt` for consistent formatting
- Address all `cargo clippy` warnings
- Write clear, self-documenting code
- Add comments for complex logic
- Keep functions focused and testable

### Testing

- Write integration tests for new features
- Ensure all existing tests pass
- Test edge cases and error conditions
- Use the test fixtures framework in `tests/fixtures.rs`

Example test:
```rust
#[test]
fn test_new_feature() {
    let (temp_dir, project_root) = setup_test_dir();

    // Create test structure
    create_file(&project_root, "test.md", "content");

    // Run tree2md
    let output = run_tree2md(&[project_root.to_str().unwrap()]);

    // Assert expectations
    assert!(output.contains("test.md"));
}
```
