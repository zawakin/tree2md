mod fixtures;

use assert_cmd::Command;
use fixtures::{p, FixtureBuilder};
use std::path::Path;

/// Run tree2md from `cwd` and return stdout.
fn run_in(cwd: &Path, args: &[String]) -> String {
    let assert = Command::cargo_bin("tree2md")
        .unwrap()
        .current_dir(cwd)
        .args(args)
        .assert()
        .success();
    String::from_utf8_lossy(&assert.get_output().stdout).to_string()
}

/// File-type counts are kept distinct (Rust 4, Markdown 3, Ignore 2) so the
/// "By type" order does not depend on tie-breaking.
fn fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    FixtureBuilder::new()
        .dir(".git")
        .file(".gitignore", "*.log\n")
        .file("README.md", "# readme\n")
        .file("app.log", "log\n")
        .file("src/main.rs", "fn main() {}\n")
        .file("src/lib.rs", "pub fn lib() {}\n")
        .file("src/util.rs", "pub fn util() {}\n")
        .file("src/gen/.gitignore", "*.tmp\n")
        .file("src/gen/out.tmp", "tmp\n")
        .file("src/gen/keep.rs", "fn keep() {}\n")
        .file("docs/guide.md", "# guide\n")
        .file("docs/faq.md", "# faq\n")
        .build()
}

fn variants() -> Vec<Vec<String>> {
    vec![
        vec![],
        vec!["-I".into(), "*.rs".into()],
        vec!["-X".into(), "docs".into()],
        vec!["-c".into()],
    ]
}

/// Matching and content reads must not depend on how TARGET is spelled.
/// A relative target (the default `.`) used to be matched by canonicalizing
/// every entry; it is now matched by its logical path, which must give the
/// same result as an absolute target.
#[test]
fn test_relative_target_matches_absolute_target() {
    let (_tmp, root) = fixture();

    for extra in variants() {
        let mut rel_args = extra.clone();
        rel_args.push(".".into());
        let mut abs_args = extra.clone();
        abs_args.push(p(&root));

        let rel = run_in(&root, &rel_args);
        let abs = run_in(&root, &abs_args);
        assert_eq!(rel, abs, "args {:?}", extra);
    }

    // Spot-check that filtering actually ran.
    let out = run_in(&root, &[".".into()]);
    assert!(out.contains("main.rs"));
    assert!(out.contains("keep.rs"));
    assert!(!out.contains("app.log"), "root .gitignore applies");
    assert!(!out.contains("out.tmp"), "nested .gitignore applies");
}

#[test]
fn test_relative_subdir_target_matches_absolute_target() {
    let (_tmp, root) = fixture();

    for target in ["src", "./src", "docs/../src"] {
        for extra in variants() {
            let mut rel_args = extra.clone();
            rel_args.push(target.into());
            let mut abs_args = extra.clone();
            abs_args.push(p(root.join("src")));

            let rel = run_in(&root, &rel_args);
            let abs = run_in(&root, &abs_args);
            assert_eq!(rel, abs, "target {:?}, args {:?}", target, extra);
        }
    }

    // Content mode reads files through the resolved path.
    let out = run_in(&root, &["-c".into(), "src".into()]);
    assert!(out.contains("pub fn lib() {}"), "got:\n{}", out);
}
