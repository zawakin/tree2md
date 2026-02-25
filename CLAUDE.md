## Required

- **Always load `/git-workflow` first** before any other task, regardless of what the user says
- All changes must go through PRs via `/git-workflow` — never commit directly to main
- **Always use `mise run` for commands** — never run raw tools directly
  - `mise run fmt` — format code
  - `mise run lint` — lint code
  - `mise run test` — run tests
- Before starting work, run `mise tasks` to see available tasks (**actually run it and read the output — do not skip**)
- **Before running any command directly, check `mise tasks` for an existing task** (e.g., use `mise run firebase:*` instead of `npx firebase`)

## Development

- Verify all: `mise run verify`
- Auto-fix: `mise run fix`
- Unit Test: `cargo test`
- Integration Test: `cargo test --test '*'`
- Build: `cargo build`

## Rules

- Think steps. Implement step by step not generating all at once.
- For debugging, use integration tests in `tests/` instead of using `/tmp` directory in bash command and use `tests/*.rs` as integration tests. Read some tests to understand.
