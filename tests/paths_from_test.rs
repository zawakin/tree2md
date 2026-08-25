mod fixtures;

use assert_cmd::Command;
use fixtures::{p, run_tree2md, FixtureBuilder};
use std::fs;

fn projects_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    FixtureBuilder::new()
        .file("README.md", "# Root")
        .file("projects/001_alpha/project.json", r#"{"status":"active"}"#)
        .file("projects/001_alpha/notes.md", "alpha")
        .file("projects/002_beta/project.json", r#"{"status":"done"}"#)
        .file("projects/002_beta/notes.md", "beta")
        .file("projects/003_gamma/project.json", r#"{"status":"active"}"#)
        .file("projects/003_gamma/src/main.rs", "fn main() {}")
        .file("projects/003_gamma/build/out.bin", "bin")
        .build()
}

#[test]
fn paths_from_file_lists_files() {
    let (_tmp, root) = projects_fixture();
    let list = root.join("list.txt");
    fs::write(
        &list,
        format!(
            "{}\n{}\n",
            root.join("projects/001_alpha/notes.md").display(),
            root.join("README.md").display()
        ),
    )
    .unwrap();

    let (out, err, ok) = run_tree2md([p(&root), "--paths-from".into(), p(&list)]);
    assert!(ok, "stderr: {err}");
    assert!(out.contains("README.md"));
    assert!(out.contains("notes.md"));
    assert!(out.contains("001_alpha"), "ancestor dir should be shown");
    assert!(!out.contains("project.json"), "unlisted sibling excluded");
    assert!(!out.contains("002_beta"));
    assert!(!out.contains("003_gamma"));
    assert!(
        !out.contains("list.txt"),
        "the list file itself is not listed"
    );
}

#[test]
fn listed_directory_includes_its_contents() {
    let (_tmp, root) = projects_fixture();
    let list = root.join("list.txt");
    fs::write(
        &list,
        format!("{}/\n", root.join("projects/003_gamma").display()),
    )
    .unwrap();

    let (out, _, ok) = run_tree2md([p(&root), "--paths-from".into(), p(&list)]);
    assert!(ok);
    assert!(out.contains("003_gamma"));
    assert!(out.contains("main.rs"));
    assert!(out.contains("project.json"));
    assert!(!out.contains("001_alpha"));
    assert!(!out.contains("README.md"));
}

#[test]
fn files0_from_stdin_with_relative_paths() {
    let (_tmp, root) = projects_fixture();
    // Relative paths are resolved against the process cwd, like fd/find output.
    let input = "projects/001_alpha\0projects/003_gamma/src/main.rs\0";
    let assert = Command::cargo_bin("tree2md")
        .unwrap()
        .current_dir(&root)
        .args(["--files0-from", "-"])
        .write_stdin(input)
        .assert()
        .success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    assert!(out.contains("001_alpha"));
    assert!(out.contains("notes.md"));
    assert!(out.contains("main.rs"));
    assert!(!out.contains("002_beta"));
    assert!(
        !out.contains("out.bin"),
        "unlisted sibling of main.rs excluded"
    );
}

#[test]
fn paths_from_with_target_subdir_and_cwd_relative_entries() {
    let (_tmp, root) = projects_fixture();
    let assert = Command::cargo_bin("tree2md")
        .unwrap()
        .current_dir(&root)
        .args(["projects", "--paths-from", "-"])
        .write_stdin("projects/002_beta/notes.md\nREADME.md\n")
        .assert()
        .success();
    let out = String::from_utf8_lossy(&assert.get_output().stdout).to_string();
    let err = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
    assert!(out.contains("002_beta"));
    assert!(out.contains("notes.md"));
    assert!(!out.contains("001_alpha"));
    assert!(
        err.contains("outside the scanned root"),
        "README.md is outside TARGET and should be warned about: {err}"
    );
}

#[test]
fn exclude_rules_carve_out_listed_paths() {
    let (_tmp, root) = projects_fixture();
    let list = root.join("list.txt");
    fs::write(
        &list,
        format!("{}\n", root.join("projects/003_gamma").display()),
    )
    .unwrap();

    let (out, _, ok) = run_tree2md([
        p(&root),
        "--paths-from".into(),
        p(&list),
        "-X".into(),
        "build".into(),
    ]);
    assert!(ok);
    assert!(out.contains("main.rs"));
    assert!(
        !out.contains("out.bin"),
        "-X build prunes inside a listed dir"
    );
}

#[test]
fn listed_path_overrides_safety_filter() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("node_modules/pkg/index.js", "x")
        .file("src/a.js", "a")
        .build();
    let list = root.join("list.txt");
    fs::write(
        &list,
        format!("{}\n", root.join("node_modules/pkg/index.js").display()),
    )
    .unwrap();

    let (out, _, ok) = run_tree2md([p(&root), "--paths-from".into(), p(&list)]);
    assert!(ok);
    assert!(
        out.contains("index.js"),
        "explicitly listed path is shown even under node_modules"
    );
    assert!(!out.contains("a.js"));
}

#[test]
fn nonexistent_entries_are_ignored() {
    let (_tmp, root) = projects_fixture();
    let list = root.join("list.txt");
    fs::write(
        &list,
        format!(
            "{}\n{}\n",
            root.join("does/not/exist.txt").display(),
            root.join("README.md").display()
        ),
    )
    .unwrap();
    let (out, _, ok) = run_tree2md([p(&root), "--paths-from".into(), p(&list)]);
    assert!(ok);
    assert!(out.contains("README.md"));
    assert!(!out.contains("does"));
}

#[test]
fn missing_list_file_is_an_error() {
    let (_tmp, root) = projects_fixture();
    let (_, err, ok) = run_tree2md([p(&root), "--paths-from".into(), p(root.join("nope.txt"))]);
    assert!(!ok);
    assert!(err.contains("cannot read path list"), "stderr: {err}");
}

#[test]
fn paths_from_and_files0_from_conflict() {
    let (_tmp, root) = projects_fixture();
    let (_, _, ok) = run_tree2md([
        p(&root),
        "--paths-from".into(),
        "-".into(),
        "--files0-from".into(),
        "-".into(),
    ]);
    assert!(!ok);
}
