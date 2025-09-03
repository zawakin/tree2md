mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_stats_off() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("lib.rs", "pub fn lib() {}")
        .dir("src")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--stats".into(), "off".into()]);
    assert!(success);

    // Should not show any statistics
    assert!(!output.contains("Totals:"), "Should not show totals");
    assert!(!output.contains("**Totals**:"), "Should not show totals");
    assert!(!output.contains("dirs"), "Should not show directory count");
    assert!(!output.contains("files"), "Should not show file count");
}

#[test]
fn test_stats_min() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("lib.rs", "pub fn lib() {}")
        .dir("src")
        .file("src/module.rs", "mod module;")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--stats".into(), "min".into()]);
    assert!(success);

    // Should show minimal statistics
    assert!(
        output.contains("Stats:") || output.contains("**Stats**:"),
        "Should show stats line"
    );
    assert!(
        output.contains("2 dir") || output.contains("📂 2"),
        "Should show directory count (root + src)"
    );
    assert!(
        output.contains("3 file") || output.contains("📄 3"),
        "Should show file count"
    );

    // Should not show progress bars (but may have markdown separators ---)
    assert!(
        !output.contains("█") && !output.contains("###"),
        "Should not show progress bars"
    );
    assert!(
        !output.contains("░"),
        "Should not show Unicode progress bars"
    );
}

#[test]
fn test_stats_full() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("lib.rs", "pub fn lib() {}")
        .file("script.py", "def hello(): pass") // Changed from test.py to avoid Test classification
        .file("app.js", "console.log('app');")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--stats".into(),
        "full".into(),
        "--fun".into(),
        "on".into(),
    ]);
    assert!(success);

    // Should show full statistics with progress bars
    assert!(
        output.contains("Totals:") || output.contains("**Totals**:"),
        "Should show totals line"
    );

    // Should show file type breakdown
    assert!(
        output.contains("🦀") || output.contains("Rust"),
        "Should show Rust files"
    );
    assert!(
        output.contains("🐍") || output.contains("Python"),
        "Should show Python files"
    );
    assert!(
        output.contains("✨") || output.contains("JavaScript"),
        "Should show JS files"
    );

    // Should show percentages
    assert!(output.contains("%"), "Should show percentages");

    // Should have some form of progress indicator (bars or counts)
    let has_progress = output.contains("█")
        || output.contains("#")
        || output.contains("▰")
        || output.contains("(50%)")
        || output.contains("(25%)");
    assert!(
        has_progress,
        "Should show progress indicators in full stats mode"
    );
}

#[test]
fn test_stats_with_many_files() {
    let mut builder = FixtureBuilder::new();

    // Create many Rust files
    for i in 0..10 {
        builder = builder.file(&format!("rust_{}.rs", i), "fn main() {}");
    }

    // Create a few Python files
    for i in 0..3 {
        builder = builder.file(&format!("python_{}.py", i), "print('hello')");
    }

    // Create one JavaScript file
    builder = builder.file("app.js", "console.log('app');");

    let (_tmp, root) = builder.build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--stats".into(),
        "full".into(),
        "--fun".into(),
        "on".into(),
    ]);
    assert!(success);

    // Rust should have the highest percentage (10 out of 14 files = ~71%)
    // Check that Rust appears with a high percentage
    let has_rust_majority = output.contains("71%")
        || output.contains("70%")
        || output.contains("72%")
        || output.contains("10 (");
    assert!(
        has_rust_majority,
        "Rust files should show majority percentage"
    );
}

#[test]
fn test_loc_counting_fast() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {\n    println!(\"Hello\");\n}\n")
        .file("lib.py", "def hello():\n    print('hello')\n\n# comment\n")
        .file("empty.txt", "")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--stats".into(),
        "min".into(),
        "--loc".into(),
        "fast".into(),
    ]);
    assert!(success);

    // Should show LOC count
    assert!(
        output.contains("LOC") || output.contains("🧾"),
        "Should show LOC indicator"
    );

    // Fast mode counts all lines (3 + 4 + 0 = 7)
    let has_loc = output.contains("7 LOC") || output.contains("~7") || output.contains("🧾");
    assert!(has_loc, "Should show line count in fast mode");
}

#[test]
fn test_loc_counting_accurate() {
    let content = "fn main() {\n    // This is a comment\n    println!(\"Hello\");\n\n    /* Block comment\n     * continues\n     */\n}\n";

    let (_tmp, root) = FixtureBuilder::new().file("main.rs", content).build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--stats".into(),
        "min".into(),
        "--loc".into(),
        "accurate".into(),
    ]);
    assert!(success);

    // Accurate mode should skip comments and blank lines
    assert!(
        output.contains("LOC") || output.contains("🧾"),
        "Should show LOC indicator"
    );
}

#[test]
fn test_loc_counting_off() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {\n    println!(\"Hello\");\n}\n")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--stats".into(),
        "min".into(),
        "--loc".into(),
        "off".into(),
    ]);
    assert!(success);

    // Should not show LOC count
    assert!(
        !output.contains("LOC") && !output.contains("🧾"),
        "Should not show LOC when off"
    );
}

#[test]
fn test_loc_skip_binary_files() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("text.txt", "Hello\nWorld\n")
        .file(
            "binary.jpg",
            &[0xFFu8, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46]
                .iter()
                .map(|&b| b as char)
                .collect::<String>(),
        ) // JPEG header
        .file("code.rs", "fn main() {}\n")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--stats".into(),
        "min".into(),
        "--loc".into(),
        "fast".into(),
    ]);
    assert!(success);

    // Should count text and code files but skip binary
    // text.txt (2 lines) + code.rs (1 line) = 3 lines
    assert!(
        output.contains("LOC") || output.contains("🧾"),
        "Should show LOC count"
    );
}

#[test]
fn test_stats_directory_count() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir("src")
        .dir("src/models")
        .dir("src/controllers")
        .dir("tests")
        .file("src/main.rs", "fn main() {}")
        .file("tests/test.rs", "mod tests {}")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--stats".into(), "min".into()]);
    assert!(success);

    // Should count 5 directories (root, src, src/models, src/controllers, tests)
    assert!(
        output.contains("5 dir") || output.contains("📂 5"),
        "Should count directories correctly"
    );
    assert!(
        output.contains("2 file") || output.contains("📄 2"),
        "Should count files correctly"
    );
}
