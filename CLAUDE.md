## Development
- Unit Test: `cargo test`
- Tests with Integration Test: `cargo test --test '*'`
- Build: `cargo build`
- Format: `cargo fmt`
- Lint: `cargo clippy -- -D warnings`

## Rules
- For debugging, use integration tests in `tests/` instead of using `/tmp` directory in bash command and use `tests/*.rs` as integration tests. Read some tests to understand.
