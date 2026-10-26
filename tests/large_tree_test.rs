mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

const DIRS: usize = 5000;

/// Upper bound for a single run over the fixture below. Linear assembly
/// finishes in a few seconds even in debug builds; the old per-directory
/// scan of every collected node took 15–40 seconds at this size.
const TIME_LIMIT: Duration = Duration::from_secs(10);

/// Create `pkg/dNNNN/{a.txt,b.txt,sub/c.txt}` for DIRS directories plus a
/// small `src/` with Rust files.
fn build_wide_tree(root: &Path) {
    for d in 0..DIRS {
        let dir = root.join(format!("pkg/d{:04}", d));
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("a.txt"), "").unwrap();
        fs::write(dir.join("b.txt"), "").unwrap();
        fs::write(dir.join("sub/c.txt"), "").unwrap();
    }
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "").unwrap();
    fs::write(root.join("src/main.rs"), "").unwrap();
}

#[test]
fn test_wide_tree_is_complete_and_ordered() {
    let (_tmp, root) = FixtureBuilder::new().build();
    build_wide_tree(&root);

    let start = Instant::now();
    let (output, _, success) = run_tree2md([p(&root)]);
    let elapsed = start.elapsed();
    assert!(success);
    assert!(
        elapsed < TIME_LIMIT,
        "tree2md took {:?} for {} dirs (limit {:?})",
        elapsed,
        DIRS,
        TIME_LIMIT
    );

    // Every directory and file is present.
    assert_eq!(output.matches("a.txt").count(), DIRS);
    assert_eq!(output.matches("b.txt").count(), DIRS);
    assert_eq!(output.matches("c.txt").count(), DIRS);
    assert_eq!(output.matches("sub/").count(), DIRS);

    // Directories come before files, and siblings sort by name.
    let pos = |needle: &str| {
        output
            .find(needle)
            .unwrap_or_else(|| panic!("missing {}", needle))
    };
    assert!(pos("pkg/") < pos("src/"));
    assert!(pos("d0000/") < pos("d0001/"));
    assert!(pos("lib.rs") < pos("main.rs"));

    // Within d0000: sub/ (dir) first, then a.txt, b.txt.
    let d0 = pos("d0000/");
    let d1 = pos("d0001/");
    let block = &output[d0..d1];
    let sub = block.find("sub/").unwrap();
    let c = block.find("c.txt").unwrap();
    let a = block.find("a.txt").unwrap();
    let b = block.find("b.txt").unwrap();
    assert!(sub < c && c < a && a < b, "unexpected order:\n{}", block);
}

#[test]
fn test_generic_include_over_wide_tree_is_fast() {
    // `-I '*.rs'` can match anywhere, so every directory is collected before
    // empty ones are dropped. That must not make assembly quadratic.
    let (_tmp, root) = FixtureBuilder::new().build();
    build_wide_tree(&root);

    let start = Instant::now();
    let (output, _, success) = run_tree2md([p(&root), "-I".into(), "*.rs".into()]);
    let elapsed = start.elapsed();
    assert!(success);
    assert!(
        elapsed < TIME_LIMIT,
        "tree2md -I '*.rs' took {:?} for {} dirs (limit {:?})",
        elapsed,
        DIRS,
        TIME_LIMIT
    );

    assert!(output.contains("lib.rs"));
    assert!(output.contains("main.rs"));
    assert!(!output.contains("pkg/"), "empty dirs should be removed");
    assert!(!output.contains(".txt"));
}
