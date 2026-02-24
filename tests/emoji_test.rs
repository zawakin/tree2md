mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_emoji_disabled() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("script.py", "print('hello')")
        .build();

    // Test with fun mode disabled
    let (output, _, success) = run_tree2md([p(&root), "--fun".into(), "off".into()]);
    assert!(success);

    // No emojis should be present
    assert!(
        !output.contains("🦀"),
        "Should not show Rust emoji when disabled"
    );
    assert!(
        !output.contains("🐍"),
        "Should not show Python emoji when disabled"
    );
}

#[test]
fn test_pipe_mode_no_emoji() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("script.py", "print('hello')")
        .build();

    // Pipe mode (non-TTY, the default in tests) should never show emojis
    // even with --fun on
    let (output, _, success) = run_tree2md([p(&root), "--fun".into(), "on".into()]);
    assert!(success);

    // Pipe output should be plain (no emojis)
    assert!(!output.contains("🦀"), "Pipe mode should not show emojis");
    assert!(!output.contains("🐍"), "Pipe mode should not show emojis");
    // But filenames should still appear
    assert!(output.contains("main.rs"));
    assert!(output.contains("script.py"));
}
