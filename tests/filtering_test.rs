mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

/// Test the filtering precedence: include > exclude > gitignore > safe
#[test]
fn test_precedence_include_wins_over_all() {
    let (_tmp, root) = FixtureBuilder::new()
        .file(".env", "SECRET=123") // Excluded by safe
        .file("temp.log", "log") // Will be in .gitignore
        .file("test.tmp", "tmp") // Will be in exclude pattern
        .file("main.rs", "fn main() {}")
        .file(".gitignore", "*.log\n")
        .build();

    // Test: Include pattern should win over everything
    let (output, _, success) = run_tree2md([
        p(&root),
        "-I".into(),
        ".env".into(), // Include .env (beats safe mode)
        "-I".into(),
        "*.log".into(), // Include .log (beats gitignore)
        "-I".into(),
        "*.tmp".into(), // Include .tmp (beats exclude)
        "-I".into(),
        "*.rs".into(), // Include .rs files
        "-X".into(),
        "*.tmp".into(), // Exclude .tmp (but include wins)
    ]);
    assert!(success);

    // All explicitly included files should be present
    assert!(output.contains(".env"), "Include should override safe mode");
    assert!(
        output.contains("temp.log"),
        "Include should override gitignore"
    );
    assert!(
        output.contains("test.tmp"),
        "Include should override exclude"
    );
    assert!(output.contains("main.rs"), "Should include .rs files");
}

#[test]
fn test_precedence_exclude_wins_over_gitignore() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("included.txt", "in")
        .file("excluded.txt", "ex")
        .file("normal.txt", "normal")
        .file(".gitignore", "*.log\n")
        .build();

    // Test: Exclude pattern should win over gitignore
    let (output, _, success) = run_tree2md([p(&root), "-X".into(), "excluded.txt".into()]);
    assert!(success);

    // excluded.txt should be excluded
    assert!(!output.contains("excluded.txt"));
    assert!(output.contains("included.txt"));
    assert!(output.contains("normal.txt"));
}

#[test]
fn test_precedence_gitignore_wins_over_safe() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("custom.secret", "secret")
        .file("normal.txt", "normal")
        .file(".gitignore", "*.secret\n")
        .dir(".git") // Make it a git repo so auto mode respects .gitignore
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Should be excluded by gitignore (even though safe mode wouldn't exclude it)
    assert!(
        !output.contains("custom.secret"),
        "Gitignore should exclude .secret files"
    );
    assert!(output.contains("normal.txt"));
}

#[test]
fn test_level_depth_limiting() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("level0.txt", "0")
        .file("dir1/level1.txt", "1")
        .file("dir1/dir2/level2.txt", "2")
        .file("dir1/dir2/dir3/level3.txt", "3")
        .build();

    // Test depth limit of 3 (to see up to dir2 and its contents)
    let (output, stderr, success) = run_tree2md([p(&root), "-L".into(), "3".into()]);
    assert!(success);

    // Debug output
    if !output.contains("level2.txt") {
        eprintln!("Output:\n{}", output);
        eprintln!("Stderr:\n{}", stderr);
    }

    // Should include up to level 3 (depth 2 from root)
    assert!(output.contains("level0.txt"));
    assert!(output.contains("dir1"));
    assert!(output.contains("level1.txt"));
    assert!(output.contains("dir2"));
    assert!(output.contains("level2.txt"));

    // Should NOT include level 4 (depth 3 from root)
    assert!(!output.contains("dir3"));
    assert!(!output.contains("level3.txt"));
}

#[test]
fn test_use_gitignore_modes() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("normal.txt", "normal")
        .file("ignored.log", "log")
        .file(".gitignore", "*.log\n")
        .dir(".git") // Make it a git repo
        .build();

    // Test auto mode (default) - should respect .gitignore
    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);
    assert!(output.contains("normal.txt"));
    assert!(
        !output.contains("ignored.log"),
        "Auto mode should respect .gitignore in git repo"
    );

    // Test never mode - should ignore .gitignore
    // Need --unsafe to see .log files (safe mode excludes them by default)
    let (output, _, success) = run_tree2md([
        p(&root),
        "--use-gitignore".into(),
        "never".into(),
        "--unsafe".into(),
    ]);
    assert!(success);

    assert!(output.contains("normal.txt"));
    assert!(
        output.contains("ignored.log"),
        "Never mode should not respect .gitignore"
    );

    // Test always mode - should respect .gitignore even without .git
    let (_tmp2, root2) = FixtureBuilder::new()
        .file("normal.txt", "normal")
        .file("ignored.log", "log")
        .file(".gitignore", "*.log\n")
        // No .git directory here
        .build();

    let (output, _, success) = run_tree2md([p(&root2), "--use-gitignore".into(), "always".into()]);
    assert!(success);
    assert!(output.contains("normal.txt"));
    assert!(
        !output.contains("ignored.log"),
        "Always mode should respect .gitignore even without .git"
    );
}

#[test]
fn test_glob_pattern_matching() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "main")
        .file("lib.rs", "lib")
        .file("test.js", "test")
        .file("src/app.rs", "app")
        .file("src/util.rs", "util")
        .file("src/index.js", "index")
        .build();

    // Test include with glob patterns
    let (output, _, success) = run_tree2md([
        p(&root),
        "-I".into(),
        "*.rs".into(), // Include all .rs files in root
        "-I".into(),
        "src/**/*.rs".into(), // Include all .rs files in src/
    ]);
    assert!(success);

    // Should include .rs files
    assert!(output.contains("main.rs"));
    assert!(output.contains("lib.rs"));
    assert!(output.contains("app.rs"));
    assert!(output.contains("util.rs"));

    // Should exclude .js files (not in include patterns)
    assert!(!output.contains("test.js"));
    assert!(!output.contains("index.js"));
}

#[test]
fn test_directory_pruning() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("keep/important.txt", "important")
        .file("skip/unneeded.txt", "unneeded")
        .file("skip/nested/deep.txt", "deep")
        .file("root.txt", "root")
        .build();

    // Exclude entire skip directory
    let (output, _, success) = run_tree2md([p(&root), "-X".into(), "skip/**".into()]);
    assert!(success);

    // Should include keep directory and root
    assert!(output.contains("keep"));
    assert!(output.contains("important.txt"));
    assert!(output.contains("root.txt"));

    // Should not include skip directory or its contents
    assert!(!output.contains("skip"));
    assert!(!output.contains("unneeded.txt"));
    assert!(!output.contains("deep.txt"));
}

#[test]
fn test_multiple_include_patterns() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("doc1.md", "doc1")
        .file("doc2.md", "doc2")
        .file("src/code.rs", "code")
        .file("test.txt", "test")
        .file("config.json", "config")
        .file("data.xml", "data")
        .build();

    // Multiple include patterns
    let (output, _, success) = run_tree2md([
        p(&root),
        "-I".into(),
        "*.md".into(),
        "-I".into(),
        "*.rs".into(),
        "-I".into(),
        "*.json".into(),
    ]);
    assert!(success);

    // Should include matched patterns
    assert!(output.contains("doc1.md"));
    assert!(output.contains("doc2.md"));
    assert!(output.contains("code.rs"));
    assert!(output.contains("config.json"));

    // Should exclude non-matched
    assert!(!output.contains("test.txt"));
    assert!(!output.contains("data.xml"));
}

#[test]
fn test_complex_precedence_scenario() {
    let (_tmp, root) = FixtureBuilder::new()
        .file(".env", "secret") // Excluded by safe
        .file("build/output.js", "out") // Will be in .gitignore
        .file("temp.txt", "temp") // Will be in exclude
        .file("important.log", "log") // Will be in .gitignore but included
        .file("src/main.rs", "main") // Normal file
        .file(".gitignore", "build/\n*.log\n")
        .dir(".git") // Make it a git repo
        .build();

    // Complex scenario with all precedence levels
    let (output, _, success) = run_tree2md([
        p(&root),
        "-I".into(),
        "*.log".into(), // Include .log files (overrides gitignore)
        "-I".into(),
        "**/*.rs".into(), // Include .rs files
        "-X".into(),
        "temp.txt".into(), // Exclude temp.txt
                           // Safe mode is on by default
    ]);
    assert!(success);

    // Include pattern wins
    assert!(output.contains("important.log"));

    // Exclude pattern works
    assert!(!output.contains("temp.txt"));

    // Gitignore works (not overridden)
    assert!(!output.contains("build"));

    // Safe mode works (not overridden)
    assert!(!output.contains(".env"));

    // Normal files included
    assert!(output.contains("main.rs"));
}
