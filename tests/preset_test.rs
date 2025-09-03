mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_preset_readme() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {\n    println!(\"Hello\");\n}\n")
        .file("lib.rs", "pub fn lib() {}")
        .dir("src")
        .file("src/module.rs", "mod module;")
        .build();

    // Test readme preset
    let (output, _, success) = run_tree2md([p(&root), "--preset".into(), "readme".into()]);
    assert!(success);

    // Readme preset should:
    // - Use markdown output format (bullet lists, with links enabled by default)
    assert!(
        output.contains("- [main.rs]") || output.contains("- main.rs"),
        "Should use markdown bullet format"
    );

    // - Show full stats
    assert!(
        output.contains("Totals:") || output.contains("**Totals**:"),
        "Should show statistics"
    );

    // - Not show fun features (emojis)
    assert!(
        !output.contains("🦀"),
        "Should not show emojis in readme preset"
    );

    // - Show LOC count (fast mode)
    assert!(
        output.contains("LOC") || output.contains("lines"),
        "Should show line count"
    );
}

#[test]
fn test_preset_fun() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("script.py", "def hello(): pass") // Changed from test.py to avoid Test classification
        .dir("docs")
        .build();

    // Test fun preset
    let (output, _, success) = run_tree2md([p(&root), "--preset".into(), "fun".into()]);
    assert!(success);

    // Fun preset should:
    // - Enable all fun features (emojis)
    assert!(
        output.contains("🦀"),
        "Should show Rust emoji in fun preset"
    );
    assert!(
        output.contains("🐍"),
        "Should show Python emoji in fun preset"
    );
    assert!(
        output.contains("📁"),
        "Should show directory emoji in fun preset"
    );

    // - Show full stats
    assert!(
        output.contains("Totals:")
            || output.contains("**Totals**:")
            || output.contains("By type:")
            || output.contains("**By type**:"),
        "Should show detailed statistics"
    );
}

#[test]
fn test_preset_light() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("lib.rs", "pub fn lib() {}")
        .build();

    // Test light preset
    let (output, _, success) = run_tree2md([p(&root), "--preset".into(), "light".into()]);
    assert!(success);

    // Light preset should:
    // - Show minimal stats
    assert!(
        output.contains("Totals:")
            || output.contains("**Totals**:")
            || output.contains("2 file")
            || output.contains("📄 2"),
        "Should show minimal statistics"
    );

    // - Not show progress bars
    assert!(
        !output.contains("█") && !output.contains("###"),
        "Should not show progress bars"
    );
}

#[test]
fn test_preset_override() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .build();

    // Note: Currently presets override explicit flags (this is a known limitation)
    // Put the explicit flag BEFORE the preset to test if it gets overridden
    let (output, _, success) = run_tree2md([
        p(&root),
        "--fun".into(),
        "on".into(),
        "--preset".into(),
        "readme".into(), // readme preset will override and disable fun mode
    ]);
    assert!(success);

    // Due to current implementation, preset overrides explicit flags
    // So readme preset's fun=off will win
    assert!(
        !output.contains("🦀"),
        "Preset currently overrides explicit flags (known limitation)"
    );
}

#[test]
fn test_preset_with_custom_emoji() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .build();

    // Test fun preset with custom emoji
    let (output, _, success) = run_tree2md([
        p(&root),
        "--preset".into(),
        "fun".into(),
        "--emoji".into(),
        ".rs=🚀".into(),
    ]);
    assert!(success);

    // Should use custom emoji instead of default
    assert!(output.contains("🚀"), "Should use custom emoji");
    assert!(!output.contains("🦀"), "Should not use default emoji");
}

#[test]
fn test_preset_with_depth_limit() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir("level1")
        .dir("level1/level2")
        .dir("level1/level2/level3")
        .file("level1/file1.txt", "1")
        .file("level1/level2/file2.txt", "2")
        .file("level1/level2/level3/file3.txt", "3")
        .build();

    // Test preset with depth limit
    let (output, _, success) = run_tree2md([
        p(&root),
        "--preset".into(),
        "light".into(),
        "-L".into(),
        "2".into(),
    ]);
    assert!(success);

    // With -L 2, the output should show level1 directory
    // Files inside level1 like file1.txt won't be shown if depth limit prevents traversal
    assert!(output.contains("level1"), "Should show level1 directory");

    // Files at deeper levels should not be included
    assert!(
        !output.contains("file2.txt"),
        "Should not include file at depth 3"
    );
    assert!(
        !output.contains("file3.txt"),
        "Should not include file at depth 4"
    );
}

#[test]
fn test_no_preset_default_behavior() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .build();

    // Test default behavior without preset
    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Default should show the tree without special features
    assert!(output.contains("main.rs"), "Should show files");
}
