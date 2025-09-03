mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_emoji_default_mapping() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("script.py", "print('hello')")
        .file("app.js", "console.log('hello')")
        .file("README.md", "# README")
        .file("Makefile", "all:\n\techo hello")
        .dir("src")
        .build();

    // Test with fun mode enabled
    let (output, _, success) = run_tree2md([p(&root), "--fun".into(), "on".into()]);
    assert!(success);

    // Check for default emojis
    assert!(
        output.contains("🦀"),
        "Should show Rust emoji for .rs files"
    );
    assert!(
        output.contains("🐍"),
        "Should show Python emoji for .py files"
    );
    assert!(
        output.contains("✨"),
        "Should show JavaScript emoji for .js files"
    );
    assert!(
        output.contains("📘"),
        "Should show Markdown emoji for .md files"
    );
    assert!(output.contains("🔨"), "Should show Makefile emoji");
    assert!(output.contains("📁"), "Should show directory emoji");
}

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
fn test_emoji_custom_override() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("script.py", "print('hello')")
        .build();

    // Test with custom emoji overrides
    let (output, _, success) = run_tree2md([
        p(&root),
        "--fun".into(),
        "on".into(),
        "--emoji".into(),
        ".rs=🚀".into(),
        "--emoji".into(),
        ".py=🔥".into(),
    ]);
    assert!(success);

    // Check for custom emojis
    assert!(output.contains("🚀"), "Should show custom Rust emoji");
    assert!(output.contains("🔥"), "Should show custom Python emoji");
    assert!(!output.contains("🦀"), "Should not show default Rust emoji");
    assert!(
        !output.contains("🐍"),
        "Should not show default Python emoji"
    );
}

#[test]
fn test_emoji_pattern_override() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("test_main.rs", "fn main() {}")
        .file("main_test.py", "def test_main(): pass")
        .file("regular.rs", "fn regular() {}")
        .build();

    // Test with pattern-based emoji override
    let (output, _, success) = run_tree2md([
        p(&root),
        "--fun".into(),
        "on".into(),
        "--emoji".into(),
        "test=🧪".into(),
    ]);
    assert!(success);

    // Test files should have test emoji
    assert!(
        output.contains("🧪"),
        "Should show test emoji for test files"
    );
    // Regular files should keep default emoji
    assert!(
        output.contains("🦀"),
        "Should show default Rust emoji for non-test files"
    );
}

#[test]
fn test_emoji_in_markdown_output() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("app.py", "print('hello')")
        .dir("src")
        .build();

    // Test markdown output mode with emojis
    let (output, _, success) = run_tree2md([
        p(&root),
        "--output".into(),
        "md".into(),
        "--fun".into(),
        "on".into(),
    ]);
    assert!(success);

    // Should have markdown bullet list format with emojis
    // With links enabled by default, files have Markdown links
    assert!(
        output.contains("🦀 [main.rs]") || output.contains("- 🦀 main.rs"),
        "Should have emoji in markdown list"
    );
    assert!(
        output.contains("🐍 [app.py]") || output.contains("- 🐍 app.py"),
        "Should have emoji in markdown list"
    );
    assert!(
        output.contains("- 📁 src/"),
        "Should have directory emoji in markdown"
    );
}

#[test]
fn test_emoji_special_files() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("LICENSE", "MIT License")
        .file(".gitignore", "*.log")
        .file("Cargo.lock", "# lock file")
        .file("Dockerfile", "FROM rust")
        .file("test_suite.rs", "mod tests {}")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--fun".into(), "on".into()]);
    assert!(success);

    // Check special file emojis
    assert!(output.contains("📜"), "Should show license emoji");
    assert!(output.contains("🗂"), "Should show ignore file emoji");
    assert!(output.contains("📦"), "Should show lock file emoji");
    assert!(output.contains("🐳"), "Should show Docker emoji");
    assert!(output.contains("🧪"), "Should show test file emoji");
}
