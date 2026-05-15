//! Regression tests for the -I / -X last-match-wins overhaul (#0.10).
//!
//! Scenarios cover the patterns that previously needed many retries to
//! get the intended output, because the original priority rules silently
//! ignored `-X` after a path-specific `-I`.

mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

fn monorepo_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    FixtureBuilder::new()
        .file("projects/alpha/index.ts", "x")
        .file("projects/alpha/build/out.js", "x")
        .file("projects/alpha/build_map.json", "x")
        .file("projects/alpha/data.generated.json", "x")
        .file("projects/beta/index.ts", "x")
        .file("projects/archived/old.ts", "x")
        .file("packages/pkg-core/src/nested/foo.ts", "x")
        .file("packages/pkg-core/src/nested/foo.test.ts", "x")
        .file("packages/pkg-core/src/feature_x/bar.ts", "x")
        .file("packages/pkg-engine/src/lib.ts", "x")
        .file("src/main.rs", "x")
        .file("tests/t1.rs", "x")
        .build()
}

#[test]
fn exclude_bare_name_works_anywhere() {
    let (_t, root) = monorepo_fixture();
    let (out, _, ok) = run_tree2md([p(&root), "-X".into(), "archived".into()]);
    assert!(ok);
    assert!(!out.contains("old.ts"));
}

#[test]
fn exclude_narrows_path_specific_include() {
    let (_t, root) = monorepo_fixture();
    let (out, _, ok) = run_tree2md([
        p(&root),
        "-I".into(),
        "projects/alpha/**".into(),
        "-X".into(),
        "build".into(),
    ]);
    assert!(ok);
    assert!(out.contains("index.ts"));
    assert!(!out.contains("out.js"), "-X build must prune build/out.js");
}

#[test]
fn exclude_with_trailing_slash_equivalent_to_bare() {
    let (_t, root) = monorepo_fixture();
    let (out, _, ok) = run_tree2md([
        p(&root),
        "-I".into(),
        "projects/alpha/**".into(),
        "-X".into(),
        "build/".into(),
    ]);
    assert!(ok);
    assert!(!out.contains("out.js"));
}

#[test]
fn exclude_double_star_extension_silently_fixed() {
    let (_t, root) = monorepo_fixture();
    let (out, _, ok) = run_tree2md([
        p(&root),
        "-I".into(),
        "projects/alpha/**".into(),
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
    let (_t, root) = monorepo_fixture();
    for pat in ["*.test.ts", "**.test.ts", "**/*.test.ts"] {
        let (out, _, ok) = run_tree2md([
            p(&root),
            "-I".into(),
            "packages/pkg-core/**.ts".into(),
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
    let (_t, root) = monorepo_fixture();
    let (out, _, ok) = run_tree2md([
        p(&root),
        "-X".into(),
        "packages".into(),
        "-I".into(),
        "packages/pkg-engine/**/*.ts".into(),
    ]);
    assert!(ok);
    assert!(out.contains("lib.ts"));
    assert!(!out.contains("bar.ts"));
    assert!(!out.contains("foo.ts"));
}

#[test]
fn brace_expansion_with_exclude() {
    let (_t, root) = monorepo_fixture();
    let (out, _, ok) = run_tree2md([
        p(&root),
        "-I".into(),
        "packages/pkg-{core,engine}/**.ts".into(),
        "-X".into(),
        "nested".into(),
    ]);
    assert!(ok);
    assert!(out.contains("bar.ts"));
    assert!(out.contains("lib.ts"));
    assert!(!out.contains("foo.ts"), "nested dir should be pruned");
    assert!(!out.contains("foo.test.ts"));
}

/// Regression for v0.10.0: `fix_double_star` and `brace_expand` iterated
/// over `bytes()` and re-encoded each UTF-8 continuation byte as its own
/// `char`, garbling multi-byte directory names. The result was that
/// `-I "projects/日本語/**"` silently matched nothing.
#[test]
fn utf8_path_pattern_matches() {
    let (_t, root) = FixtureBuilder::new()
        .file("projects/日本語ディレクトリ/a.ts", "x")
        .file("projects/日本語ディレクトリ/b.ts", "x")
        .file("projects/other/c.ts", "x")
        .build();

    let (out, _, ok) = run_tree2md([
        p(&root),
        "-I".into(),
        "projects/日本語ディレクトリ/**".into(),
    ]);
    assert!(ok);
    assert!(out.contains("a.ts"), "a.ts should be included: {}", out);
    assert!(out.contains("b.ts"));
    assert!(!out.contains("c.ts"));
}

#[test]
fn utf8_path_with_brace_pattern() {
    let (_t, root) = FixtureBuilder::new()
        .file("packages/日本-core/x.ts", "x")
        .file("packages/日本-engine/y.ts", "x")
        .file("packages/other/z.ts", "x")
        .build();

    let (out, _, ok) = run_tree2md([
        p(&root),
        "-I".into(),
        "packages/日本-{core,engine}/**".into(),
    ]);
    assert!(ok);
    assert!(out.contains("x.ts"));
    assert!(out.contains("y.ts"));
    assert!(!out.contains("z.ts"));
}

#[test]
fn include_subtree_prunes_unrelated_siblings() {
    let (_t, root) = monorepo_fixture();
    let (out, _, ok) = run_tree2md([p(&root), "-I".into(), "projects/alpha/**".into()]);
    assert!(ok);
    assert!(out.contains("index.ts"));
    assert!(!out.contains("beta"));
    assert!(!out.contains("archived"));
}
