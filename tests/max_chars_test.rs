mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_max_chars_head_truncates() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("a.rs", "fn a() {\n    println!(\"aaaa\");\n}\n")
        .file("b.rs", "fn b() {\n    println!(\"bbbb\");\n}\n")
        .build();

    // Very small budget so truncation must happen
    let (output, _, success) =
        run_tree2md([p(&root), "-c".into(), "--max-chars".into(), "30".into()]);
    assert!(success);

    // Should contain file section headers
    assert!(output.contains("## "));
    // Should contain truncation marker
    assert!(
        output.contains("lines omitted)"),
        "Should show omitted lines marker when truncated: {}",
        output
    );
}

#[test]
fn test_max_chars_head_no_truncation_when_fits() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("small.rs", "fn main() {}\n")
        .build();

    // Large budget — no truncation needed
    let (output, _, success) =
        run_tree2md([p(&root), "-c".into(), "--max-chars".into(), "100000".into()]);
    assert!(success);

    assert!(output.contains("fn main()"));
    assert!(
        !output.contains("lines omitted)"),
        "Should not truncate when content fits within budget"
    );
}

#[test]
fn test_max_chars_nest_mode() {
    let content = "fn main() {\n    if true {\n        if nested {\n            deeply_nested();\n            more_nested();\n            even_more();\n        }\n    }\n}\n";
    let (_tmp, root) = FixtureBuilder::new().file("deep.rs", content).build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "-c".into(),
        "--max-chars".into(),
        "80".into(),
        "--contents-mode".into(),
        "nest".into(),
    ]);
    assert!(success);

    // Should still contain the top-level structure
    assert!(output.contains("fn main()"));
    // Should have collapsed some deeply nested lines
    assert!(
        output.contains("... ("),
        "Nest mode should collapse deeply indented blocks: {}",
        output
    );
}

#[test]
fn test_max_chars_without_contents_flag_fails() {
    let (_tmp, root) = FixtureBuilder::new().file("a.rs", "fn a() {}\n").build();

    // --max-chars without -c should fail
    let (_output, stderr, success) = run_tree2md([p(&root), "--max-chars".into(), "100".into()]);
    assert!(
        !success,
        "Should fail when --max-chars is used without -c: stderr={}",
        stderr
    );
}

#[test]
fn test_max_chars_budget_proportional() {
    // Create files with very different sizes
    let small = "x\n";
    let large = "line\n".repeat(100);

    let (_tmp, root) = FixtureBuilder::new()
        .file("small.txt", small)
        .file("large.txt", &large)
        .build();

    let (output, _, success) =
        run_tree2md([p(&root), "-c".into(), "--max-chars".into(), "100".into()]);
    assert!(success);

    // The large file should be truncated more
    assert!(output.contains("## "));
    // Both files should appear in output
    assert!(output.contains("small.txt"));
    assert!(output.contains("large.txt"));
}

#[test]
fn test_contents_mode_default_is_head() {
    let content = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\n";
    let (_tmp, root) = FixtureBuilder::new().file("data.txt", content).build();

    // Without --contents-mode, default should be head
    let (output, _, success) =
        run_tree2md([p(&root), "-c".into(), "--max-chars".into(), "20".into()]);
    assert!(success);

    // Head mode keeps from the beginning
    assert!(output.contains("line1"));
}
