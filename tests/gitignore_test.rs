mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

/// Nested .gitignore files should be respected.
/// A .gitignore in a subdirectory should only affect files in that subtree.
#[test]
fn test_nested_gitignore_in_subdirectory() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file(".gitignore", "*.log\n")
        .file("root.txt", "root")
        .file("root.log", "root log")
        .file("sub/sub.txt", "sub text")
        .file("sub/sub.log", "sub log")
        .file("sub/generated/out.txt", "generated")
        .file("sub/.gitignore", "generated/\n")
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // root.log excluded by root .gitignore
    assert!(
        !output.contains("root.log"),
        "root .gitignore should exclude *.log"
    );
    // sub/sub.log also excluded by root .gitignore
    assert!(
        !output.contains("sub.log"),
        "root .gitignore should exclude nested *.log"
    );
    // sub/generated/ excluded by sub/.gitignore
    assert!(
        !output.contains("generated"),
        "nested .gitignore should exclude generated/"
    );
    // Normal files should be included
    assert!(output.contains("root.txt"));
    assert!(output.contains("sub.txt"));
}

/// Deeply nested .gitignore should work at arbitrary depth.
#[test]
fn test_deeply_nested_gitignore() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file("a/b/c/.gitignore", "*.tmp\n")
        .file("a/b/c/keep.txt", "keep")
        .file("a/b/c/remove.tmp", "remove")
        .file("a/b/keep.txt", "keep")
        .file("a/b/keep.tmp", "keep tmp at b level")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--unsafe".into()]);
    assert!(success);

    // .tmp should be excluded only under a/b/c/, not at a/b/
    assert!(
        !output.contains("remove.tmp"),
        "a/b/c/.gitignore should exclude *.tmp in c/"
    );
    assert!(
        output.contains("keep.tmp"),
        "a/b/.tmp should NOT be excluded by c/.gitignore"
    );
    assert!(output.contains("keep.txt"));
}

/// Auto-detection should find .git in ancestor directories.
/// Running tree2md on a subdirectory of a git repo should still respect gitignore.
#[test]
fn test_auto_detection_from_subdirectory() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file(".gitignore", "*.secret\n")
        .file("sub/visible.txt", "visible")
        .file("sub/hidden.secret", "secret")
        .build();

    // Run tree2md targeting the subdirectory
    let sub_path = root.join("sub");
    let (output, _, success) = run_tree2md([p(&sub_path)]);
    assert!(success);

    assert!(output.contains("visible.txt"));
    assert!(
        !output.contains("hidden.secret"),
        "Auto mode should detect .git in ancestor and respect .gitignore"
    );
}

/// Include patterns (-I) should override nested .gitignore.
#[test]
fn test_include_overrides_nested_gitignore() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file("sub/.gitignore", "*.generated\n")
        .file("sub/keep.rs", "code")
        .file("sub/output.generated", "generated output")
        .build();

    // Without -I, .generated should be excluded
    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);
    assert!(
        !output.contains("output.generated"),
        "Without -I, nested gitignore should exclude .generated"
    );

    // With -I, .generated should be included (include overrides gitignore)
    let (output, _, success) = run_tree2md([
        p(&root),
        "-I".into(),
        "*.generated".into(),
        "-I".into(),
        "*.rs".into(),
    ]);
    assert!(success);
    assert!(
        output.contains("output.generated"),
        "Include pattern should override nested .gitignore"
    );
    assert!(output.contains("keep.rs"));
}

/// When --use-gitignore=never, nested .gitignore should have no effect.
#[test]
fn test_never_mode_ignores_nested_gitignore() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file("sub/.gitignore", "*.data\n")
        .file("sub/file.data", "data")
        .file("sub/file.txt", "text")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--use-gitignore".into(),
        "never".into(),
        "--unsafe".into(),
    ]);
    assert!(success);
    assert!(
        output.contains("file.data"),
        "Never mode should not respect any .gitignore"
    );
    assert!(output.contains("file.txt"));
}
