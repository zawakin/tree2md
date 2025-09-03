mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_level_shows_dirs_at_max_depth() {
    // Test that --level N shows directories at depth N but not files at depth N
    // This is the edge case: directories should be visible at the max depth level
    let (_tmp, root) = FixtureBuilder::new()
        .file("root.txt", "root")
        .dir("L1")
        .file("L1/file1.txt", "level 1 file")
        .dir("L1/L2")
        .file("L1/L2/file2.txt", "level 2 file")
        .dir("L1/L2/L3")
        .file("L1/L2/L3/file3.txt", "level 3 file")
        .dir("L1/L2/L3/L4")
        .file("L1/L2/L3/L4/file4.txt", "level 4 file")
        .build();

    // Test --level 1: should show L1/ but not its contents
    let (output, _, success) = run_tree2md([p(&root), "--level".into(), "1".into()]);
    assert!(success);
    assert!(output.contains("root.txt"), "L1: Should show root.txt");
    assert!(output.contains("L1/"), "L1: Should show L1 directory");
    assert!(
        !output.contains("file1.txt"),
        "L1: Should NOT show file1.txt"
    );
    assert!(!output.contains("L2/"), "L1: Should NOT show L2 directory");

    // Test --level 2: should show L1/, L2/ and file1.txt
    let (output, _, success) = run_tree2md([p(&root), "--level".into(), "2".into()]);
    assert!(success);
    assert!(output.contains("root.txt"), "L2: Should show root.txt");
    assert!(output.contains("L1/"), "L2: Should show L1 directory");
    assert!(output.contains("file1.txt"), "L2: Should show file1.txt");
    assert!(output.contains("L2/"), "L2: Should show L2 directory");
    assert!(
        !output.contains("file2.txt"),
        "L2: Should NOT show file2.txt"
    );
    assert!(!output.contains("L3/"), "L2: Should NOT show L3 directory");

    // Test --level 3: should show L1/, L2/, L3/, file1.txt, file2.txt
    let (output, _, success) = run_tree2md([p(&root), "--level".into(), "3".into()]);
    assert!(success);
    assert!(output.contains("root.txt"), "L3: Should show root.txt");
    assert!(output.contains("L1/"), "L3: Should show L1 directory");
    assert!(output.contains("file1.txt"), "L3: Should show file1.txt");
    assert!(output.contains("L2/"), "L3: Should show L2 directory");
    assert!(output.contains("file2.txt"), "L3: Should show file2.txt");
    assert!(output.contains("L3/"), "L3: Should show L3 directory");
    assert!(
        !output.contains("file3.txt"),
        "L3: Should NOT show file3.txt"
    );
    assert!(!output.contains("L4/"), "L3: Should NOT show L4 directory");
}

#[test]
fn test_level_absolute_depth_enforcement() {
    // This test verifies that --level enforces an absolute depth limit from the root,
    // not a relative depth from any "leaf" directory
    let (_tmp, root) = FixtureBuilder::new()
        .dir("project")
        .file("project/main.rs", "fn main() {}")
        .dir("project/src")
        .file("project/src/lib.rs", "pub fn lib() {}")
        .dir("project/src/modules")
        .file("project/src/modules/mod1.rs", "mod1")
        .dir("project/src/modules/submodules")
        .file("project/src/modules/submodules/sub1.rs", "sub1")
        .build();

    // With --level 3, we enforce an absolute depth limit:
    // - project/ (depth 1) - shown
    // - project/src/ (depth 2) - shown
    // - project/src/modules/ (depth 3) - shown (directory at max depth)
    // - project/src/modules/submodules/ (depth 4) - NOT shown (exceeds limit)
    let (output, _, success) = run_tree2md([p(&root), "--level".into(), "3".into()]);
    assert!(success);

    // Directories up to and including depth 3
    assert!(
        output.contains("project/"),
        "Should show project directory at depth 1"
    );
    assert!(
        output.contains("src/"),
        "Should show src directory at depth 2"
    );
    assert!(
        output.contains("modules/"),
        "Should show modules directory at depth 3"
    );

    // Files shown only up to depth 2 (files at depth 3 are not shown)
    assert!(output.contains("main.rs"), "Should show main.rs at depth 1");
    assert!(output.contains("lib.rs"), "Should show lib.rs at depth 2");
    assert!(
        !output.contains("mod1.rs"),
        "Should NOT show mod1.rs at depth 3"
    );

    // Content at depth 4 should NOT be shown (absolute depth limit)
    assert!(
        !output.contains("submodules/"),
        "Should NOT show submodules/ at depth 4 (exceeds --level 3)"
    );
    assert!(
        !output.contains("sub1.rs"),
        "Should NOT show sub1.rs at depth 4"
    );
}
