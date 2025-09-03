mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_include_dotfile_pattern() {
    let (_tmp, root) = FixtureBuilder::new()
        .file(".env", "SECRET=123")
        .file(".env.local", "LOCAL=456")
        .file(".gitignore", "*.log")
        .file("main.rs", "fn main() {}")
        .build();

    // Test: -I .env should match .env file specifically, not a directory
    let (output, _, success) =
        run_tree2md([p(&root), "-I".into(), ".env".into(), "--unsafe".into()]);
    assert!(success);

    // Should include .env file
    assert!(output.contains(".env"), "Should include .env file");

    // Should NOT include other files
    assert!(
        !output.contains(".env.local"),
        "Should not include .env.local"
    );
    assert!(
        !output.contains(".gitignore"),
        "Should not include .gitignore"
    );
    assert!(!output.contains("main.rs"), "Should not include main.rs");
}

#[test]
fn test_include_extension_pattern() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("lib.rs", "pub fn lib() {}")
        .file("test.js", "test")
        .file("README.md", "# Readme")
        .build();

    // Test: -I *.rs should match all .rs files
    let (output, _, success) = run_tree2md([p(&root), "-I".into(), "*.rs".into()]);
    assert!(success);

    // Should include .rs files
    assert!(output.contains("main.rs"), "Should include main.rs");
    assert!(output.contains("lib.rs"), "Should include lib.rs");

    // Should NOT include other files
    assert!(!output.contains("test.js"), "Should not include test.js");
    assert!(
        !output.contains("README.md"),
        "Should not include README.md"
    );
}

#[test]
fn test_include_directory_name_pattern() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("src/main.rs", "fn main() {}")
        .file("src/lib.rs", "pub fn lib() {}")
        .file("test/test.rs", "test")
        .file("README.md", "# Readme")
        .build();

    // Test: -I src should match everything under src directory
    let (output, _, success) = run_tree2md([p(&root), "-I".into(), "src".into()]);
    assert!(success);

    // Should include files under src
    assert!(output.contains("main.rs"), "Should include src/main.rs");
    assert!(output.contains("lib.rs"), "Should include src/lib.rs");

    // Should NOT include files outside src
    assert!(
        !output.contains("test.rs"),
        "Should not include test/test.rs"
    );
    assert!(
        !output.contains("README.md"),
        "Should not include README.md"
    );
}
