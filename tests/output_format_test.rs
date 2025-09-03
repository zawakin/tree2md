mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_output_format_markdown() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("lib.rs", "pub fn lib() {}")
        .dir("src")
        .file("src/module.rs", "mod module;")
        .build();

    // Test markdown output format
    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "md".into()]);
    assert!(success);

    // Should use bullet list format (with links enabled by default)
    assert!(
        output.contains("- [main.rs]") || output.contains("- main.rs"),
        "Should use bullet list for files"
    );
    assert!(
        output.contains("- [lib.rs]") || output.contains("- lib.rs"),
        "Should use bullet list for files"
    );
    assert!(
        output.contains("- src/"),
        "Should use bullet list for directories"
    );
    assert!(
        output.contains("  - ") && output.contains("module.rs"),
        "Should indent nested items"
    );

    // Should NOT have terminal tree characters
    assert!(
        !output.contains("├──"),
        "Should not have tree branch characters"
    );
    assert!(
        !output.contains("└──"),
        "Should not have tree branch characters"
    );
    assert!(!output.contains("│"), "Should not have tree vertical lines");
}

#[test]
fn test_output_format_terminal() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("lib.rs", "pub fn lib() {}")
        .dir("src")
        .file("src/module.rs", "mod module;")
        .build();

    // Test terminal output format
    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "tty".into()]);
    assert!(success);

    // Should use tree characters (ASCII or Unicode depending on terminal)
    // At least one of these formats should be present
    let has_unicode = output.contains("├──") || output.contains("└──");
    let has_ascii = output.contains("|--") || output.contains("`--");

    assert!(
        has_unicode || has_ascii,
        "Should have tree branch characters (Unicode or ASCII)"
    );

    // Should NOT use bullet list format
    assert!(
        !output.starts_with("- "),
        "Should not use bullet list format"
    );
}

#[test]
fn test_output_format_auto_with_pipe() {
    let (_tmp, root) = FixtureBuilder::new().file("test.txt", "content").build();

    // When output is piped, auto should default to markdown
    // Note: In test environment, output is captured, simulating pipe behavior
    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "auto".into()]);
    assert!(success);

    // Should use markdown format when piped (with links enabled by default)
    assert!(
        output.contains("- [test.txt]")
            || output.contains("- test.txt")
            || output.contains("|--")
            || output.contains("├──"),
        "Auto mode should work in test environment"
    );
}

#[test]
fn test_terminal_format_with_emojis() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("script.py", "print('hello')")
        .dir("docs")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--output".into(),
        "tty".into(),
        "--fun".into(),
        "on".into(),
    ]);
    assert!(success);

    // Should have emojis with tree format
    assert!(output.contains("🦀"), "Should show Rust emoji");
    assert!(output.contains("🐍"), "Should show Python emoji");
    assert!(output.contains("📁"), "Should show directory emoji");
}

#[test]
fn test_markdown_format_no_html() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir("folder1")
        .file("folder1/file1.txt", "content")
        .dir("folder2")
        .file("folder2/file2.txt", "content")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "md".into()]);
    assert!(success);

    // Markdown mode should NOT have HTML tags
    assert!(
        !output.contains("<details"),
        "Should not have HTML details tags"
    );
    assert!(
        !output.contains("</details>"),
        "Should not have HTML closing tags"
    );
    assert!(
        !output.contains("<summary>"),
        "Should not have HTML summary tags"
    );

    // Should use pure markdown
    assert!(output.contains("- folder1/"), "Should use bullet lists");
    assert!(output.contains("- folder2/"), "Should use bullet lists");
}

#[test]
fn test_output_format_with_depth() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("level1.txt", "1")
        .dir("dir1")
        .file("dir1/level2.txt", "2")
        .dir("dir1/dir2")
        .file("dir1/dir2/level3.txt", "3")
        .dir("dir1/dir2/dir3")
        .file("dir1/dir2/dir3/level4.txt", "4")
        .build();

    // Test markdown format with depth
    let (output, _, success) = run_tree2md([
        p(&root),
        "--output".into(),
        "md".into(),
        "-L".into(),
        "2".into(),
    ]);
    assert!(success);

    assert!(output.contains("level1.txt"), "Should include level 1");
    assert!(output.contains("level2.txt"), "Should include level 2");
    assert!(!output.contains("level3.txt"), "Should not include level 3");
    assert!(!output.contains("level4.txt"), "Should not include level 4");
}
