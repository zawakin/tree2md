#![cfg(unix)]
//! Symlink handling: skipped by default, traversed with --follow-links.
//!
//! Symlinks point at content that lives *outside* the scanned target so the
//! followed entries are unambiguously distinct from real, directly-reachable
//! files (the builder canonicalizes paths, which would otherwise collapse an
//! in-tree symlink onto its target's name).

mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};
use std::os::unix::fs::symlink;

/// Build a fixture with a `scanned/` target plus a sibling `shared/` directory
/// reachable only through symlinks, and return the path to `scanned/`.
fn fixture_with_symlinks() -> (tempfile::TempDir, std::path::PathBuf) {
    let (tmp, root) = FixtureBuilder::new()
        .file("scanned/visible.txt", "directly reachable")
        .file("shared/alpha.txt", "reached via link_dir or link_file")
        .file("shared/beta.txt", "reached only via link_dir traversal")
        .build();

    // scanned/link_file -> ../shared/alpha.txt   (a symlinked file)
    symlink("../shared/alpha.txt", root.join("scanned/link_file")).expect("symlink file");
    // scanned/link_dir  -> ../shared              (a symlinked directory)
    symlink("../shared", root.join("scanned/link_dir")).expect("symlink dir");

    let scanned = root.join("scanned");
    (tmp, scanned)
}

#[test]
fn test_symlinks_skipped_by_default() {
    let (_tmp, scanned) = fixture_with_symlinks();

    let (output, _, success) = run_tree2md([p(&scanned)]);
    assert!(success);

    // The real file is listed.
    assert!(
        output.contains("visible.txt"),
        "expected visible.txt in output, got:\n{}",
        output
    );

    // Nothing reached through a symlink should appear.
    assert!(
        !output.contains("link_file"),
        "symlinked file entry should be skipped, got:\n{}",
        output
    );
    assert!(
        !output.contains("link_dir"),
        "symlinked dir entry should be skipped, got:\n{}",
        output
    );
    assert!(
        !output.contains("alpha.txt"),
        "symlink target should not be reached by default, got:\n{}",
        output
    );
    assert!(
        !output.contains("beta.txt"),
        "content behind a symlinked dir should not be reached by default, got:\n{}",
        output
    );
}

#[test]
fn test_follow_links_traverses_symlinks() {
    let (_tmp, scanned) = fixture_with_symlinks();

    let (output, _, success) = run_tree2md([p(&scanned), "--follow-links".into()]);
    assert!(success);

    assert!(
        output.contains("visible.txt"),
        "expected visible.txt in output, got:\n{}",
        output
    );

    // The symlinked file resolves to alpha.txt.
    assert!(
        output.contains("alpha.txt"),
        "symlinked file should be followed, got:\n{}",
        output
    );

    // beta.txt is reachable ONLY by descending into the symlinked directory,
    // so its presence proves directory traversal (not just file resolution).
    assert!(
        output.contains("beta.txt"),
        "content behind a symlinked dir should be reached with --follow-links, got:\n{}",
        output
    );
}

#[test]
fn test_follow_links_cycle_terminates() {
    // A symlink that points back at its own parent forms a cycle. The walker
    // must detect the loop and terminate instead of recursing forever.
    let (_tmp, root) = FixtureBuilder::new().file("a/keep.txt", "content").build();
    // a/loop -> ..  (points at the scanned root, i.e. an ancestor)
    symlink("..", root.join("a/loop")).expect("symlink loop");

    let (output, _, success) = run_tree2md([p(&root), "--follow-links".into()]);
    assert!(success, "follow-links over a cycle should still succeed");
    assert!(output.contains("keep.txt"), "got:\n{}", output);
}

#[test]
fn test_follow_links_short_flag() {
    let (_tmp, scanned) = fixture_with_symlinks();

    let (output, _, success) = run_tree2md([p(&scanned), "-l".into()]);
    assert!(success);

    assert!(
        output.contains("beta.txt"),
        "-l should behave like --follow-links, got:\n{}",
        output
    );
}
