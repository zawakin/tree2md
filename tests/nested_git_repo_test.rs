mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

/// Nested git worktrees (directories with a .git file) should be pruned.
/// This prevents worktree contents from leaking into the output.
#[test]
fn test_nested_worktree_is_pruned() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file("src/main.rs", "fn main() {}")
        .file("README.md", "# Project")
        // Simulate a git worktree: directory with a .git file (not directory)
        .file(
            ".worktrees/pool-001/.git",
            "gitdir: /tmp/fake/.git/worktrees/pool-001",
        )
        .file(".worktrees/pool-001/src/main.rs", "fn main() {}")
        .file(".worktrees/pool-001/README.md", "# Worktree copy")
        .file(
            ".worktrees/pool-002/.git",
            "gitdir: /tmp/fake/.git/worktrees/pool-002",
        )
        .file(".worktrees/pool-002/package.json", "{}")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--unsafe".into()]);
    assert!(success);

    // Root project files should be present
    assert!(output.contains("src/"));
    assert!(output.contains("main.rs"));
    assert!(output.contains("README.md"));

    // Worktree directories should be completely pruned
    assert!(
        !output.contains(".worktrees"),
        "Worktree directories should be pruned, got:\n{}",
        output
    );
    assert!(
        !output.contains("pool-001"),
        "Worktree pool-001 should not appear"
    );
    assert!(
        !output.contains("pool-002"),
        "Worktree pool-002 should not appear"
    );
    assert!(
        !output.contains("package.json"),
        "Files inside worktrees should not appear"
    );
}

/// Nested git submodules (directories with a .git file) should be pruned.
#[test]
fn test_nested_submodule_is_pruned() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file("src/main.rs", "fn main() {}")
        // Simulate a git submodule
        .file("vendor/lib/.git", "gitdir: ../../.git/modules/vendor/lib")
        .file("vendor/lib/src/lib.rs", "pub fn lib() {}")
        .file("vendor/lib/README.md", "# Submodule")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--unsafe".into()]);
    assert!(success);

    assert!(output.contains("main.rs"));
    // Submodule directory should be pruned
    assert!(
        !output.contains("lib.rs"),
        "Submodule contents should not appear, got:\n{}",
        output
    );
}

/// Nested standalone git repos (.git directory) should be pruned.
#[test]
fn test_nested_git_repo_directory_is_pruned() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file("src/main.rs", "fn main() {}")
        // Simulate a nested git repo with .git directory
        .dir("third_party/dep/.git")
        .file("third_party/dep/lib.rs", "pub fn dep() {}")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--unsafe".into()]);
    assert!(success);

    assert!(output.contains("main.rs"));
    assert!(
        !output.contains("lib.rs"),
        "Nested git repo contents should not appear, got:\n{}",
        output
    );
}

/// .git/info/exclude patterns should be respected.
#[test]
fn test_git_info_exclude_is_respected() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git/info")
        .file(".git/info/exclude", ".worktrees/\nscratch/\n")
        .file("src/main.rs", "fn main() {}")
        .file(".worktrees/data.txt", "should be excluded")
        .file("scratch/notes.txt", "should be excluded")
        .file("visible/file.txt", "should be visible")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--unsafe".into()]);
    assert!(success);

    assert!(output.contains("main.rs"));
    assert!(output.contains("visible"));
    assert!(
        !output.contains("scratch"),
        ".git/info/exclude should exclude scratch/, got:\n{}",
        output
    );
}

/// When --use-gitignore=never, nested git repos should still be pruned
/// (this is a safety measure, not a gitignore feature).
#[test]
fn test_nested_git_repos_pruned_even_without_gitignore() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir(".git")
        .file("src/main.rs", "fn main() {}")
        .file(
            ".worktrees/pool/.git",
            "gitdir: /tmp/fake/.git/worktrees/pool",
        )
        .file(".worktrees/pool/leaked.txt", "should not appear")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--use-gitignore".into(),
        "never".into(),
        "--unsafe".into(),
    ]);
    assert!(success);

    assert!(output.contains("main.rs"));
    assert!(
        !output.contains("leaked.txt"),
        "Nested git repos should be pruned regardless of gitignore setting, got:\n{}",
        output
    );
}
