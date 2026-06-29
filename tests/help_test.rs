mod fixtures;

use fixtures::run_tree2md;

/// `-h` is the reflex view: it must carry the analogy and a runnable starter
/// command, and stay tight by demoting auxiliary option groups to `--help`.
#[test]
fn test_short_help_has_quick_start_and_stays_tight() {
    let (stdout, _, success) = run_tree2md(["-h"]);
    assert!(success);

    // Purpose + on-ramp live in -h (Rules 2 & 6).
    assert!(stdout.contains("optimized for AI agents"));
    assert!(stdout.contains("QUICK START:"));
    assert!(stdout.contains("tree2md -c | pbcopy"));

    // Core options stay visible.
    assert!(stdout.contains("--contents"));
    assert!(stdout.contains("--include"));
    assert!(stdout.contains("--max-chars"));
    assert!(stdout.contains("--unsafe"));

    // Auxiliary groups are demoted to --help (Rule 12).
    assert!(!stdout.contains("Fun & Style"));
    assert!(!stdout.contains("--emoji"));
    assert!(!stdout.contains("Statistics"));
    assert!(!stdout.contains("--stats"));
    assert!(!stdout.contains("--contents-mode"));
}

/// `--help` keeps the full reference: the demoted options must reappear.
#[test]
fn test_long_help_restores_demoted_options() {
    let (stdout, _, success) = run_tree2md(["--help"]);
    assert!(success);

    assert!(stdout.contains("Fun & Style"));
    assert!(stdout.contains("--emoji"));
    assert!(stdout.contains("Statistics"));
    assert!(stdout.contains("--stats"));
    assert!(stdout.contains("--loc"));
    assert!(stdout.contains("--contents-mode"));
    assert!(stdout.contains("--safe"));
}

/// A usage error must end with runnable next commands, not just clap's footer
/// (Rule 13).
#[test]
fn test_unknown_flag_prints_actionable_try_block() {
    let (_, stderr, success) = run_tree2md(["--nonexistent"]);
    assert!(!success);

    // Keeps clap's existing "similar argument" tip.
    assert!(stderr.contains("similar argument"));
    // Adds runnable next commands.
    assert!(stderr.contains("Try:"));
    assert!(stderr.contains("tree2md -c | pbcopy"));
}

/// Help and version are not errors: they exit 0 and skip the "Try:" block.
#[test]
fn test_help_does_not_print_try_block() {
    let (_, stderr, success) = run_tree2md(["--help"]);
    assert!(success);
    assert!(!stderr.contains("Try:"));
}
