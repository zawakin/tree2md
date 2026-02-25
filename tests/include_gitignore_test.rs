mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

// =============================================================================
// Include patterns should respect gitignore for directories they don't
// explicitly target. e.g., `-I src` should NOT descend into gitignored
// directories like `target/` just because they might contain a `src/` subdir.
// =============================================================================

/// `-I src` should not include `target/doc/src/` when `target` is gitignored.
/// The generic include `**/src/**` must not override gitignore pruning of `target/`.
#[test]
fn test_include_dir_respects_gitignore_pruning() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file(".gitignore", "target/\n")
        .file("src/main.rs", "fn main() {}")
        .file("src/lib.rs", "pub fn lib() {}")
        .file("target/doc/src/main.rs.html", "<html>main</html>")
        .file("target/doc/src/lib.rs.html", "<html>lib</html>")
        .file("README.md", "# Root")
        .build();

    // -I src should include only src/, not target/doc/src/
    let (output, _, success) = run_tree2md([p(&root), "-I".into(), "src".into()]);
    assert!(success);

    // Should include files under src/
    assert!(output.contains("main.rs"), "Should include src/main.rs");
    assert!(output.contains("lib.rs"), "Should include src/lib.rs");

    // Should NOT include files under gitignored target/
    assert!(
        !output.contains("main.rs.html"),
        "Should not include target/doc/src/main.rs.html (target is gitignored)"
    );
    assert!(
        !output.contains("lib.rs.html"),
        "Should not include target/doc/src/lib.rs.html (target is gitignored)"
    );
    // The target directory itself should not appear
    assert!(
        !output.contains("target"),
        "target/ directory should be pruned by gitignore"
    );
}

/// Same test with trailing slash: `-I src/` should behave identically to `-I src`.
#[test]
fn test_include_dir_with_trailing_slash_respects_gitignore() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file(".gitignore", "target/\n")
        .file("src/main.rs", "fn main() {}")
        .file("target/doc/src/main.rs.html", "<html>main</html>")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "-I".into(), "src/".into()]);
    assert!(success);

    assert!(output.contains("main.rs"), "Should include src/main.rs");
    assert!(
        !output.contains("main.rs.html"),
        "Should not include target/doc/src/main.rs.html"
    );
    assert!(
        !output.contains("target"),
        "target/ should be pruned by gitignore"
    );
}

/// `-I *.rs` should not include .rs files inside gitignored directories.
#[test]
fn test_include_extension_respects_gitignore_pruning() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file(".gitignore", "build/\n")
        .file("src/main.rs", "fn main() {}")
        .file("src/lib.rs", "pub fn lib() {}")
        .file("build/generated/output.rs", "// generated")
        .file("README.md", "# Root")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "-I".into(), "*.rs".into()]);
    assert!(success);

    // Should include .rs files in non-gitignored dirs
    assert!(output.contains("main.rs"), "Should include src/main.rs");
    assert!(output.contains("lib.rs"), "Should include src/lib.rs");

    // Should NOT include .rs files inside gitignored build/
    assert!(
        !output.contains("output.rs"),
        "Should not include build/generated/output.rs (build is gitignored)"
    );
    assert!(
        !output.contains("build"),
        "build/ directory should be pruned by gitignore"
    );
}

/// Safety preset should also prune directories even with generic include patterns.
/// e.g., `-I src --unsafe` should still not show `.git/` contents.
#[test]
fn test_include_dir_respects_safety_pruning() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file(".gitignore", "")
        .file("src/main.rs", "fn main() {}")
        .file("node_modules/pkg/src/index.js", "// pkg source")
        .build();

    // With safe mode (default), node_modules should be excluded
    let (output, _, success) = run_tree2md([p(&root), "-I".into(), "src".into()]);
    assert!(success);

    assert!(output.contains("main.rs"), "Should include src/main.rs");
    assert!(
        !output.contains("index.js"),
        "Should not include node_modules/pkg/src/index.js (safety preset)"
    );
}
