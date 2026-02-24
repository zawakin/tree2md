mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

// =============================================================================
// Bug 1: Exclude should narrow down include results
// Currently, -I always wins over -X, which means -X cannot refine -I results.
// Expected: -X should be able to exclude files that -I includes.
// =============================================================================

#[test]
fn test_exclude_narrows_include_rs_files() {
    // Setup: project with both src and test .rs files
    let (_tmp, root) = FixtureBuilder::new()
        .file("src/main.rs", "fn main() {}")
        .file("src/lib.rs", "pub fn lib() {}")
        .file("src/test_helper.rs", "// test helper")
        .file("tests/test_main.rs", "// test")
        .file("README.md", "# Root")
        .build();

    // Include all .rs files, but exclude test_*.rs
    let (output, _, success) = run_tree2md([
        p(&root),
        "-I".into(),
        "*.rs".into(),
        "-X".into(),
        "test_*.rs".into(),
    ]);
    assert!(success);

    // Should include .rs files
    assert!(output.contains("main.rs"), "Should include main.rs");
    assert!(output.contains("lib.rs"), "Should include lib.rs");

    // Should NOT include test_*.rs files (excluded by -X)
    assert!(
        !output.contains("test_helper.rs"),
        "test_helper.rs should be excluded by -X test_*.rs"
    );
    assert!(
        !output.contains("test_main.rs"),
        "test_main.rs should be excluded by -X test_*.rs"
    );

    // Should NOT include non-.rs files
    assert!(
        !output.contains("README.md"),
        "README.md should not be included"
    );
}

#[test]
fn test_exclude_narrows_include_directory() {
    // Setup: __tests__ directory with mixed file types
    let (_tmp, root) = FixtureBuilder::new()
        .file("__tests__/test_api.py", "# api test")
        .file("__tests__/test_db.py", "# db test")
        .file("__tests__/conftest.py", "# conftest")
        .file("__tests__/__snapshots__/snap1.json", "{}")
        .file("__tests__/__snapshots__/snap2.json", "{}")
        .file("src/main.py", "# main")
        .build();

    // Include __tests__ directory, but exclude __snapshots__ subdirectory
    let (output, _, success) = run_tree2md([
        p(&root),
        "-I".into(),
        "__tests__".into(),
        "-X".into(),
        "__snapshots__".into(),
    ]);
    assert!(success);

    // Should include test files
    assert!(output.contains("test_api.py"), "Should include test_api.py");
    assert!(output.contains("test_db.py"), "Should include test_db.py");
    assert!(output.contains("conftest.py"), "Should include conftest.py");

    // Should NOT include __snapshots__ files
    assert!(
        !output.contains("snap1.json"),
        "snap1.json should be excluded by -X __snapshots__"
    );
    assert!(
        !output.contains("snap2.json"),
        "snap2.json should be excluded by -X __snapshots__"
    );

    // Should NOT include files outside __tests__
    assert!(
        !output.contains("main.py"),
        "main.py should not be included"
    );
}

// =============================================================================
// Bug 2: -X directory should not prevent -I from matching files inside it
// When -X prunes a directory, -I patterns for files inside cannot match.
// =============================================================================

#[test]
fn test_include_pattern_inside_excluded_directory() {
    // Setup: __tests__ has .py and .js files
    let (_tmp, root) = FixtureBuilder::new()
        .file("__tests__/test_api.py", "# api test")
        .file("__tests__/test_db.py", "# db test")
        .file("__tests__/setup.js", "// setup")
        .file("__tests__/nested/deep_test.py", "# deep")
        .file("src/main.py", "# main")
        .build();

    // Include only .py files from __tests__ via explicit path pattern
    let (output, _, success) = run_tree2md([p(&root), "-I".into(), "__tests__/**/*.py".into()]);
    assert!(success);

    // Should include .py files under __tests__
    assert!(
        output.contains("test_api.py"),
        "Should include __tests__/test_api.py"
    );
    assert!(
        output.contains("test_db.py"),
        "Should include __tests__/test_db.py"
    );
    assert!(
        output.contains("deep_test.py"),
        "Should include __tests__/nested/deep_test.py"
    );

    // Should NOT include non-.py files from __tests__
    assert!(
        !output.contains("setup.js"),
        "setup.js should not be included"
    );

    // Should NOT include files outside __tests__
    assert!(
        !output.contains("main.py"),
        "main.py should not be included"
    );
}

#[test]
fn test_include_specific_files_despite_directory_exclude() {
    // -X excludes the directory, but -I should still include specific files from it
    let (_tmp, root) = FixtureBuilder::new()
        .file("vendor/lib1/code.py", "# code")
        .file("vendor/lib1/README.md", "# readme")
        .file("vendor/lib2/code.py", "# code2")
        .file("vendor/lib2/LICENSE", "MIT")
        .file("src/main.py", "# main")
        .build();

    // Exclude vendor directory but include .py files from it
    let (output, _, success) = run_tree2md([
        p(&root),
        "-X".into(),
        "vendor".into(),
        "-I".into(),
        "vendor/**/*.py".into(),
    ]);
    assert!(success);

    // Should include .py files from vendor (include overrides exclude for matching files)
    assert!(
        output.contains("code.py"),
        "Should include vendor .py files"
    );

    // Should NOT include non-.py files from vendor
    assert!(
        !output.contains("README.md"),
        "README.md in vendor should be excluded"
    );
    assert!(
        !output.contains("LICENSE"),
        "LICENSE in vendor should be excluded"
    );
}

// =============================================================================
// Bug 3: Wildcard pattern consistency
// __tests__ and __tests__/** should behave identically
// __tests__/**/*.py should only match .py files
// =============================================================================

#[test]
fn test_directory_name_vs_glob_equivalence() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("__tests__/test1.py", "# test1")
        .file("__tests__/test2.js", "// test2")
        .file("__tests__/sub/test3.py", "# test3")
        .file("src/main.py", "# main")
        .build();

    // -I __tests__ (normalized to __tests__/**)
    let (output1, _, success1) = run_tree2md([p(&root), "-I".into(), "__tests__".into()]);
    assert!(success1);

    // -I __tests__/** (explicit glob)
    let (output2, _, success2) = run_tree2md([p(&root), "-I".into(), "__tests__/**".into()]);
    assert!(success2);

    // Both should include all files under __tests__
    assert!(
        output1.contains("test1.py"),
        "__tests__ should include test1.py"
    );
    assert!(
        output1.contains("test2.js"),
        "__tests__ should include test2.js"
    );
    assert!(
        output1.contains("test3.py"),
        "__tests__ should include test3.py"
    );

    assert!(
        output2.contains("test1.py"),
        "__tests__/** should include test1.py"
    );
    assert!(
        output2.contains("test2.js"),
        "__tests__/** should include test2.js"
    );
    assert!(
        output2.contains("test3.py"),
        "__tests__/** should include test3.py"
    );

    // Neither should include files outside __tests__
    assert!(
        !output1.contains("main.py"),
        "__tests__: should not include main.py"
    );
    assert!(
        !output2.contains("main.py"),
        "__tests__/**: should not include main.py"
    );
}

#[test]
fn test_directory_glob_with_extension_filter() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("__tests__/test1.py", "# test1")
        .file("__tests__/test2.js", "// test2")
        .file("__tests__/sub/test3.py", "# test3")
        .file("__tests__/sub/test4.js", "// test4")
        .file("src/main.py", "# main")
        .build();

    // -I __tests__/**/*.py should only include .py files
    let (output, _, success) = run_tree2md([p(&root), "-I".into(), "__tests__/**/*.py".into()]);
    assert!(success);

    // Should include .py files under __tests__
    assert!(output.contains("test1.py"), "Should include test1.py");
    assert!(output.contains("test3.py"), "Should include sub/test3.py");

    // Should NOT include .js files
    assert!(!output.contains("test2.js"), "Should not include test2.js");
    assert!(!output.contains("test4.js"), "Should not include test4.js");

    // Should NOT include files outside __tests__
    assert!(!output.contains("main.py"), "Should not include main.py");
}

// =============================================================================
// Bug 4: Exclude directory name should work consistently
// -X __tests__ should exclude the directory and all its contents
// =============================================================================

#[test]
fn test_exclude_directory_name_consistency() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("__tests__/test1.py", "# test1")
        .file("__tests__/sub/test2.py", "# test2")
        .file("src/main.py", "# main")
        .file("src/__tests__/nested_test.py", "# nested")
        .build();

    // -X __tests__ should exclude __tests__ at any level
    let (output, _, success) = run_tree2md([p(&root), "-X".into(), "__tests__".into()]);
    assert!(success);

    // Should NOT include any __tests__ content
    assert!(!output.contains("test1.py"), "Should not include test1.py");
    assert!(!output.contains("test2.py"), "Should not include test2.py");
    assert!(
        !output.contains("nested_test.py"),
        "Should not include nested_test.py from src/__tests__"
    );

    // Should include other files
    assert!(output.contains("main.py"), "Should include main.py");
}

#[test]
fn test_exclude_specific_extension_in_directory() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("__tests__/test1.py", "# test1")
        .file("__tests__/test2.py", "# test2")
        .file("__tests__/setup.js", "// setup")
        .file("__tests__/config.json", "{}")
        .file("src/main.py", "# main")
        .build();

    // Include __tests__ but exclude .json files
    let (output, _, success) = run_tree2md([
        p(&root),
        "-I".into(),
        "__tests__".into(),
        "-X".into(),
        "*.json".into(),
    ]);
    assert!(success);

    // Should include non-.json files under __tests__
    assert!(output.contains("test1.py"), "Should include test1.py");
    assert!(output.contains("test2.py"), "Should include test2.py");
    assert!(output.contains("setup.js"), "Should include setup.js");

    // Should NOT include .json files
    assert!(
        !output.contains("config.json"),
        "config.json should be excluded by -X *.json"
    );

    // Should NOT include files outside __tests__
    assert!(
        !output.contains("main.py"),
        "main.py should not be included"
    );
}
