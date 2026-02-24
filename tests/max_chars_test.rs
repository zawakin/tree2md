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
        run_tree2md([p(&root), "-c".into(), "--max-chars".into(), "10".into()]);
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
fn test_max_chars_head_uniform_n() {
    // All files should be truncated to the same number of lines (uniform n)
    let (_tmp, root) = FixtureBuilder::new()
        .file("a.txt", "a1\na2\na3\na4\na5\n")
        .file("b.txt", "b1\nb2\nb3\nb4\nb5\n")
        .build();

    // Budget tight enough to force truncation but allow at least 2 lines each
    // n=2: "a1\na2"(5) + "b1\nb2"(5) = 10
    // n=3: "a1\na2\na3"(8) + "b1\nb2\nb3"(8) = 16
    let (output, _, success) =
        run_tree2md([p(&root), "-c".into(), "--max-chars".into(), "12".into()]);
    assert!(success);

    // Both files should show the same omitted count (uniform n)
    let omitted_count = output.matches("(3 lines omitted)").count();
    assert_eq!(
        omitted_count, 2,
        "Both files should omit 3 lines each (uniform n=2): {}",
        output
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
fn test_max_chars_nest_uniform_threshold() {
    // Two files with similar indentation structure should be collapsed at the same threshold
    let file_a =
        "fn a() {\n    let x = 1;\n    if true {\n        deep_a1();\n        deep_a2();\n    }\n}";
    let file_b =
        "fn b() {\n    let y = 2;\n    if true {\n        deep_b1();\n        deep_b2();\n    }\n}";

    let (_tmp, root) = FixtureBuilder::new()
        .file("a.rs", file_a)
        .file("b.rs", file_b)
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "-c".into(),
        "--max-chars".into(),
        "90".into(),
        "--contents-mode".into(),
        "nest".into(),
    ]);
    assert!(success);

    // Both files should show inline collapse markers "... (N lines)"
    // (not the footer "... (N lines omitted)")
    let inline_markers: Vec<&str> = output
        .lines()
        .filter(|l| l.starts_with("... (") && !l.contains("omitted"))
        .collect();
    assert!(
        inline_markers.len() >= 2,
        "Both files should have inline collapse markers: {}",
        output
    );
    // All inline markers should have the same count (symmetric files, uniform threshold)
    let first = inline_markers[0];
    assert!(
        inline_markers.iter().all(|m| *m == first),
        "Uniform threshold should produce symmetric collapsing: {:?}",
        inline_markers
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
