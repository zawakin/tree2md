mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_include_directory_pattern() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root")
        .file("specs/test1.rs", "test1")
        .file("specs/test2.rs", "test2")
        .file("specs/nested/test3.rs", "test3")
        .file("src/main.rs", "main")
        .file("src/lib.rs", "lib")
        .build();

    // Test: -I specs should include everything under specs directory
    let (output, _, success) = run_tree2md([p(&root), "-I".into(), "specs".into()]);
    assert!(success);

    // Should include all files under specs
    assert!(output.contains("test1.rs"), "Should include specs/test1.rs");
    assert!(output.contains("test2.rs"), "Should include specs/test2.rs");
    assert!(
        output.contains("test3.rs"),
        "Should include specs/nested/test3.rs"
    );

    // Should NOT include files outside specs
    assert!(
        !output.contains("README.md"),
        "Should not include README.md"
    );
    assert!(
        !output.contains("main.rs"),
        "Should not include src/main.rs"
    );
    assert!(!output.contains("lib.rs"), "Should not include src/lib.rs");
}

#[test]
fn test_include_directory_with_glob() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root")
        .file("specs/test1.rs", "test1")
        .file("specs/test2.rs", "test2")
        .file("specs/nested/test3.rs", "test3")
        .file("src/main.rs", "main")
        .file("src/lib.rs", "lib")
        .build();

    // Test: -I specs/** should include everything under specs directory
    let (output, _, success) = run_tree2md([p(&root), "-I".into(), "specs/**".into()]);
    assert!(success);

    // Should include all files under specs
    assert!(output.contains("test1.rs"), "Should include specs/test1.rs");
    assert!(output.contains("test2.rs"), "Should include specs/test2.rs");
    assert!(
        output.contains("test3.rs"),
        "Should include specs/nested/test3.rs"
    );

    // Should NOT include files outside specs
    assert!(
        !output.contains("README.md"),
        "Should not include README.md"
    );
    assert!(
        !output.contains("main.rs"),
        "Should not include src/main.rs"
    );
    assert!(!output.contains("lib.rs"), "Should not include src/lib.rs");
}

#[test]
fn test_include_multiple_directories() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root")
        .file("specs/test.rs", "test")
        .file("tests/unit.rs", "unit")
        .file("src/main.rs", "main")
        .file("docs/guide.md", "guide")
        .build();

    // Test: -I specs -I tests should include files from both directories
    let (output, _, success) = run_tree2md([
        p(&root),
        "-I".into(),
        "specs".into(),
        "-I".into(),
        "tests".into(),
    ]);
    assert!(success);

    // Should include files from specs and tests
    assert!(output.contains("test.rs"), "Should include specs/test.rs");
    assert!(output.contains("unit.rs"), "Should include tests/unit.rs");

    // Should NOT include files from other directories
    assert!(
        !output.contains("README.md"),
        "Should not include README.md"
    );
    assert!(
        !output.contains("main.rs"),
        "Should not include src/main.rs"
    );
    assert!(
        !output.contains("guide.md"),
        "Should not include docs/guide.md"
    );
}
