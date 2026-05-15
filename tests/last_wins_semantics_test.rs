//! Regression tests for the -I / -X last-match-wins overhaul (#0.10).
//!
//! These scenarios are taken from real-world usage history where users had
//! to retry the same intent with many pattern variants because the original
//! priority rules silently ignored `-X` after a path-specific `-I`.

mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

fn user_like_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    FixtureBuilder::new()
        .file("projects/007_news/index.ts", "x")
        .file("projects/007_news/build/out.js", "x")
        .file("projects/007_news/build_map.json", "x")
        .file("projects/007_news/data.generated.json", "x")
        .file("projects/008_other/index.ts", "x")
        .file("projects/archived/old.ts", "x")
        .file("packages/othello-domain/src/online/foo.ts", "x")
        .file("packages/othello-domain/src/online/foo.test.ts", "x")
        .file("packages/othello-domain/src/aiCoach/bar.ts", "x")
        .file("packages/othello-engine/src/lib.ts", "x")
        .file("src/main.rs", "x")
        .file("tests/t1.rs", "x")
        .build()
}

#[test]
fn exclude_bare_name_works_anywhere() {
    let (_t, root) = user_like_fixture();
    let (out, _, ok) = run_tree2md([p(&root), "-X".into(), "archived".into()]);
    assert!(ok);
    assert!(!out.contains("old.ts"));
}

#[test]
fn exclude_narrows_path_specific_include() {
    let (_t, root) = user_like_fixture();
    let (out, _, ok) = run_tree2md([
        p(&root),
        "-I".into(),
        "projects/007_news/**".into(),
        "-X".into(),
        "build".into(),
    ]);
    assert!(ok);
    assert!(out.contains("index.ts"));
    assert!(!out.contains("out.js"), "-X build must prune build/out.js");
}

#[test]
fn exclude_with_trailing_slash_equivalent_to_bare() {
    let (_t, root) = user_like_fixture();
    let (out, _, ok) = run_tree2md([
        p(&root),
        "-I".into(),
        "projects/007_news/**".into(),
        "-X".into(),
        "build/".into(),
    ]);
    assert!(ok);
    assert!(!out.contains("out.js"));
}

#[test]
fn exclude_double_star_extension_silently_fixed() {
    let (_t, root) = user_like_fixture();
    let (out, _, ok) = run_tree2md([
        p(&root),
        "-I".into(),
        "projects/007_news/**".into(),
        "-X".into(),
        "**.generated.json".into(),
    ]);
    assert!(ok);
    assert!(out.contains("index.ts"));
    assert!(
        !out.contains("data.generated.json"),
        "**.generated.json must be interpreted as **/*.generated.json"
    );
}

#[test]
fn exclude_test_ts_variants_all_work() {
    let (_t, root) = user_like_fixture();
    for pat in ["*.test.ts", "**.test.ts", "**/*.test.ts"] {
        let (out, _, ok) = run_tree2md([
            p(&root),
            "-I".into(),
            "packages/othello-domain/**.ts".into(),
            "-X".into(),
            pat.into(),
        ]);
        assert!(ok, "pattern: {pat}");
        assert!(
            out.contains("bar.ts"),
            "{pat}: expected bar.ts to be kept, got: {out}"
        );
        assert!(
            !out.contains("foo.test.ts"),
            "{pat}: expected foo.test.ts to be excluded, got: {out}"
        );
    }
}

#[test]
fn include_after_exclude_carves_out_excluded_subtree() {
    let (_t, root) = user_like_fixture();
    // -X vendor -I vendor/**/*.ts style: exclude, then add a slice back.
    let (out, _, ok) = run_tree2md([
        p(&root),
        "-X".into(),
        "packages".into(),
        "-I".into(),
        "packages/othello-engine/**/*.ts".into(),
    ]);
    assert!(ok);
    assert!(out.contains("lib.ts"));
    assert!(!out.contains("bar.ts"));
    assert!(!out.contains("foo.ts"));
}

#[test]
fn brace_expansion_with_exclude() {
    let (_t, root) = user_like_fixture();
    let (out, _, ok) = run_tree2md([
        p(&root),
        "-I".into(),
        "packages/othello-{domain,engine}/**.ts".into(),
        "-X".into(),
        "online".into(),
    ]);
    assert!(ok);
    assert!(out.contains("bar.ts"));
    assert!(out.contains("lib.ts"));
    assert!(!out.contains("foo.ts"), "online dir should be pruned");
    assert!(!out.contains("foo.test.ts"));
}

#[test]
fn include_subtree_prunes_unrelated_siblings() {
    let (_t, root) = user_like_fixture();
    let (out, _, ok) = run_tree2md([p(&root), "-I".into(), "projects/007_news/**".into()]);
    assert!(ok);
    assert!(out.contains("index.ts"));
    assert!(!out.contains("008_other"));
    assert!(!out.contains("archived"));
}
