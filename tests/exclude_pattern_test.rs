mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_exclude_directory_pattern() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root")
        .file("specs/test1.rs", "test1")
        .file("specs/test2.rs", "test2")
        .file("specs/nested/test3.rs", "test3")
        .file("src/main.rs", "main")
        .file("src/lib.rs", "lib")
        .build();

    // Test: -X specs should exclude everything under specs directory
    let (output, _, success) = run_tree2md([p(&root), "-X".into(), "specs".into()]);
    assert!(success);

    // Should NOT include files under specs
    assert!(
        !output.contains("test1.rs"),
        "Should not include specs/test1.rs"
    );
    assert!(
        !output.contains("test2.rs"),
        "Should not include specs/test2.rs"
    );
    assert!(
        !output.contains("test3.rs"),
        "Should not include specs/nested/test3.rs"
    );

    // Should include files outside specs
    assert!(output.contains("README.md"), "Should include README.md");
    assert!(output.contains("main.rs"), "Should include src/main.rs");
    assert!(output.contains("lib.rs"), "Should include src/lib.rs");
}

#[test]
fn test_exclude_extension_pattern() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("lib.rs", "pub fn lib() {}")
        .file("test.js", "test")
        .file("README.md", "# Readme")
        .build();

    // Test: -X *.js should exclude all .js files
    let (output, _, success) = run_tree2md([p(&root), "-X".into(), "*.js".into()]);
    assert!(success);

    // Should include non-.js files
    assert!(output.contains("main.rs"), "Should include main.rs");
    assert!(output.contains("lib.rs"), "Should include lib.rs");
    assert!(output.contains("README.md"), "Should include README.md");

    // Should NOT include .js files
    assert!(!output.contains("test.js"), "Should not include test.js");
}

#[test]
fn test_exclude_multiple_directories() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root")
        .file("specs/test.rs", "test")
        .file("tests/unit.rs", "unit")
        .file("src/main.rs", "main")
        .file("docs/guide.md", "guide")
        .build();

    // Test: -X specs -X tests should exclude files from both directories
    let (output, _, success) = run_tree2md([
        p(&root),
        "-X".into(),
        "specs".into(),
        "-X".into(),
        "tests".into(),
    ]);
    assert!(success);

    // Should NOT include files from specs and tests
    assert!(
        !output.contains("test.rs"),
        "Should not include specs/test.rs"
    );
    assert!(
        !output.contains("unit.rs"),
        "Should not include tests/unit.rs"
    );

    // Should include files from other directories
    assert!(output.contains("README.md"), "Should include README.md");
    assert!(output.contains("main.rs"), "Should include src/main.rs");
    assert!(output.contains("guide.md"), "Should include docs/guide.md");
}

#[test]
fn test_exclude_dotfile_pattern() {
    let (_tmp, root) = FixtureBuilder::new()
        .file(".env", "SECRET=123")
        .file(".env.local", "LOCAL=456")
        .file(".gitignore", "*.log")
        .file("main.rs", "fn main() {}")
        .build();

    // Test: -X .gitignore should exclude .gitignore file specifically
    let (output, _, success) = run_tree2md([
        p(&root),
        "-X".into(),
        ".gitignore".into(),
        "--unsafe".into(),
    ]);
    assert!(success);

    // Should include other files
    assert!(output.contains(".env"), "Should include .env file");
    assert!(output.contains("main.rs"), "Should include main.rs");

    // Should NOT include .gitignore
    assert!(
        !output.contains(".gitignore"),
        "Should not include .gitignore"
    );
}
