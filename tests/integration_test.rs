use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

// --------------------------------------------------------------------------------
// Helpers
// --------------------------------------------------------------------------------

fn run_tree2md<I, S>(args: I) -> (String, String, bool)
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let mut cmd = Command::cargo_bin("tree2md").expect("tree2md binary not found");
    cmd.args(args);

    let Output {
        status,
        stdout,
        stderr,
    } = cmd.output().expect("Failed to execute tree2md");
    let stdout = String::from_utf8_lossy(&stdout).to_string();
    let stderr = String::from_utf8_lossy(&stderr).to_string();

    (stdout, stderr, status.success())
}

/// Create a "sample" like directory tree in a TempDir.
/// Returns (tempdir, root_path)
fn setup_sample_dir() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp dir");
    let root = dir.path().to_path_buf();

    // root files
    fs::write(root.join("empty.txt"), "").unwrap();
    fs::write(root.join("hello.py"), "print(\"hello\")\n").unwrap();
    fs::write(root.join("hoge.txt"), "hoge\n").unwrap();
    fs::write(
        root.join("large.txt"),
        "line1\nline2\nline3\nline4\nline5\n",
    )
    .unwrap();
    fs::write(root.join("multiline.txt"), "line1\nline2\nline3\n").unwrap();
    fs::write(root.join("no_newline.txt"), "no_newline").unwrap();

    // foo dir
    let foo = root.join("foo");
    fs::create_dir(&foo).unwrap();
    fs::write(foo.join("bar.go"), "package foo\nfunc Bar() {}\n").unwrap();
    fs::write(foo.join("bar.js"), "console.log('bar');\n").unwrap();
    fs::write(foo.join("bar.py"), "def bar():\n    pass\n").unwrap();

    (dir, root)
}

/// Create a gitignore-focused fixture like "test_gitignore".
/// Returns (tempdir, root_path)
fn setup_gitignore_fixture() -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("create temp dir");
    let root = dir.path().to_path_buf();

    // src
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(src.join("lib.rs"), "pub fn lib_function() {}\n").unwrap();

    // other files
    fs::write(root.join("README.md"), "# Readme\n").unwrap();

    // ignored dirs/files
    fs::create_dir_all(root.join("target")).unwrap();
    fs::write(root.join("target/debug.out"), "bin\n").unwrap();

    fs::create_dir_all(root.join("node_modules")).unwrap();
    fs::write(root.join("node_modules/x.js"), "module.exports=1\n").unwrap();

    fs::create_dir_all(root.join("dist")).unwrap();
    fs::write(root.join("dist/bundle.js"), "// bundled\n").unwrap();

    fs::write(root.join("test.tmp"), "tmp\n").unwrap();
    fs::write(root.join("error.log"), "log\n").unwrap();

    // local .gitignore to stabilize behavior
    fs::write(
        root.join(".gitignore"),
        "target/\nnode_modules/\ndist/\n*.tmp\n*.log\n",
    )
    .unwrap();

    (dir, root)
}

fn p<P: AsRef<Path>>(p: P) -> String {
    p.as_ref().to_string_lossy().to_string()
}

// --------------------------------------------------------------------------------
// Tests (rewritten to use temp fixtures)
// --------------------------------------------------------------------------------

#[test]
fn test_sample_directory_basic_structure() {
    let (_tmp, root) = setup_sample_dir();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    assert!(output.contains("## File Structure"));

    // files at root
    assert!(output.contains("empty.txt"));
    assert!(output.contains("hello.py"));
    assert!(output.contains("hoge.txt"));
    assert!(output.contains("large.txt"));
    assert!(output.contains("multiline.txt"));
    assert!(output.contains("no_newline.txt"));

    // foo dir and contents
    assert!(output.contains("foo/"));
    assert!(output.contains("bar.go"));
    assert!(output.contains("bar.js"));
    assert!(output.contains("bar.py"));
}

#[test]
fn test_sample_with_contents() {
    let (_tmp, root) = setup_sample_dir();

    let (output, _, success) = run_tree2md([p(&root), "--contents".into()]);
    assert!(success);

    assert!(output.contains("## File Structure"));

    assert!(output.contains("### hello.py"));
    assert!(output.contains("```python"));
    assert!(output.contains("print(\"hello\")"));

    assert!(output.contains("### foo/bar.go"));
    assert!(output.contains("```go"));
    assert!(output.contains("package foo"));
    assert!(output.contains("func Bar()"));

    assert!(output.contains("### multiline.txt"));
    assert!(output.contains("line1"));
    assert!(output.contains("line2"));
    assert!(output.contains("line3"));
}

#[test]
fn test_sample_with_extension_filter() {
    let (_tmp, root) = setup_sample_dir();

    let (output, _, success) = run_tree2md([p(&root), "--include-ext".into(), "py".into()]);
    assert!(success);

    assert!(output.contains("hello.py"));
    assert!(output.contains("bar.py"));

    assert!(!output.contains("bar.go"));
    assert!(!output.contains("bar.js"));
    assert!(!output.contains("hoge.txt"));
    assert!(!output.contains("empty.txt"));
}

#[test]
fn test_sample_multiple_extensions() {
    let (_tmp, root) = setup_sample_dir();

    let (output, _, success) = run_tree2md([p(&root), "--include-ext".into(), "py,go".into()]);
    assert!(success);

    assert!(output.contains("hello.py"));
    assert!(output.contains("bar.py"));
    assert!(output.contains("bar.go"));

    assert!(!output.contains("bar.js"));
    assert!(!output.contains("hoge.txt"));
}

#[test]
fn test_sample_with_max_lines() {
    let (_tmp, root) = setup_sample_dir();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--contents".into(),
        "--max-lines".into(),
        "2".into(),
    ]);
    assert!(success);

    assert!(output.contains("### multiline.txt"));
    assert!(output.contains("line1"));
    assert!(output.contains("line2"));
    assert!(!output.contains("line3"));
    assert!(output.contains("[Content truncated:"));
}

#[test]
fn test_sample_flat_structure() {
    let (_tmp, root) = setup_sample_dir();

    let (output, _, success) = run_tree2md([p(&root), "--flat".into()]);
    assert!(success);

    assert!(output.contains("- empty.txt"));
    assert!(output.contains("- hello.py"));
    assert!(output.contains("- foo/bar.go"));
    assert!(output.contains("- foo/bar.js"));
    assert!(output.contains("- foo/bar.py"));

    assert!(!output.contains("  -"));
    // Using auto-detected display root, so no fixed path names expected
}

#[test]
fn test_sample_no_root() {
    let (_tmp, root) = setup_sample_dir();

    // Without --root-label, root should not be shown
    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    let lines: Vec<&str> = output.lines().collect();
    let _structure_start = lines
        .iter()
        .position(|&l| l == "## File Structure")
        .unwrap();

    let root_name = root.file_name().unwrap().to_string_lossy();
    let root_line = format!("- {}/", root_name);
    assert!(
        !lines.iter().any(|l| l.trim() == root_line),
        "Root should not be shown without --root-label"
    );
}

#[test]
fn test_sample_with_root_label() {
    let (_tmp, root) = setup_sample_dir();

    let (output, _, success) = run_tree2md([p(&root), "--root-label".into(), "MyProject".into()]);
    assert!(success);

    assert!(output.contains("MyProject"));
    assert!(output.contains("empty.txt"));
    assert!(output.contains("hello.py"));
    assert!(output.contains("foo/"));
}

#[test]
fn test_sample_with_gitignore() {
    let (_tmp, root) = setup_sample_dir();

    // Add temporary .gitignore
    fs::write(root.join(".gitignore"), "*.txt\n").expect("Failed to write .gitignore");

    // Gitignore is respected by default
    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    assert!(!output.contains("empty.txt"));
    assert!(!output.contains("hoge.txt"));
    assert!(!output.contains("multiline.txt"));

    assert!(output.contains("hello.py"));
    assert!(output.contains("bar.go"));
}

#[test]
fn test_gitignore_default_behavior() {
    let (_tmp, root) = setup_gitignore_fixture();

    // Directory scan mode respects .gitignore by default
    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    assert!(output.contains("main.rs"));
    assert!(output.contains("lib.rs"));
    assert!(output.contains("README.md"));

    assert!(!output.contains("target/"));
    assert!(!output.contains("node_modules/"));
    assert!(!output.contains("dist/"));
    assert!(!output.contains("test.tmp"));
    assert!(!output.contains("error.log"));
}

#[test]
fn test_no_gitignore_flag() {
    let (_tmp, root) = setup_gitignore_fixture();

    let (output, _, success) = run_tree2md([p(&root), "--no-gitignore".into()]);
    assert!(success);

    assert!(output.contains("main.rs"));
    assert!(output.contains("lib.rs"));
    assert!(output.contains("README.md"));
    assert!(output.contains("target/"));
    assert!(output.contains("node_modules/"));
    assert!(output.contains("dist/"));
    assert!(output.contains("test.tmp"));
    assert!(output.contains("error.log"));
}

#[test]
fn test_stdin_authoritative_ignored_file() {
    use assert_cmd::Command;

    let (_tmp, root) = setup_gitignore_fixture();
    let ignored = root.join("target/debug.out");

    let assert = Command::cargo_bin("tree2md")
        .expect("bin")
        .args([p(&root), "--stdin".into()])
        .write_stdin(format!("{}\n", p(&ignored)))
        .assert();

    let output = assert
        .success()
        .stdout(predicate::str::contains("debug.out"))
        .get_output()
        .clone();

    let out = String::from_utf8_lossy(&output.stdout).to_string();
    // authoritative: no automatic directory scanning
    assert!(
        !out.contains("main.rs"),
        "Should not scan directory in authoritative mode"
    );
}

#[test]
fn test_stdin_expand_dirs_default() {
    use assert_cmd::Command;

    // Create a simple test fixture
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create files and directories
    fs::write(root.join("file1.txt"), "content1").unwrap();
    fs::write(root.join("file2.txt"), "content2").unwrap();
    fs::create_dir(root.join("ignored_dir")).unwrap();
    fs::write(root.join("ignored_dir/ignored.txt"), "ignored").unwrap();

    // Create .gitignore
    fs::write(root.join(".gitignore"), "ignored_dir/\n").unwrap();

    let assert = Command::cargo_bin("tree2md")
        .expect("bin")
        .args([p(&root), "--stdin".into(), "--expand-dirs".into()])
        .write_stdin(".\n")
        .assert();

    let output = assert.success().get_output().clone();
    let out = String::from_utf8_lossy(&output.stdout).to_string();

    // Should include normal files
    assert!(out.contains("file1.txt"));
    assert!(out.contains("file2.txt"));

    // Should NOT include ignored directory or its contents
    assert!(!out.contains("ignored_dir"));
    assert!(!out.contains("ignored.txt"));
}

#[test]
fn test_stdin_expand_dirs_no_gitignore() {
    use assert_cmd::Command;

    let (_tmp, root) = setup_gitignore_fixture();

    let assert = Command::cargo_bin("tree2md")
        .expect("bin")
        .args([
            p(&root),
            "--stdin".into(),
            "--expand-dirs".into(),
            "--no-gitignore".into(),
        ])
        .write_stdin(".\n")
        .assert();

    let output = assert.success().get_output().clone();
    let out = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(out.contains("main.rs"));
    assert!(out.contains("lib.rs"));
    assert!(out.contains("README.md"));
    assert!(out.contains("debug.out"));
    assert!(out.contains("test.tmp"));
    assert!(out.contains("error.log"));
}

#[test]
fn test_stdin_expand_ignored_dir() {
    use assert_cmd::Command;

    let (_tmp, root) = setup_gitignore_fixture();

    let assert = Command::cargo_bin("tree2md")
        .expect("bin")
        .args([p(&root), "--stdin".into(), "--expand-dirs".into()])
        .write_stdin("target\n")
        .assert();

    // When expanding an ignored directory, it should be skipped
    // resulting in "No valid files found" error
    assert
        .failure()
        .stderr(predicate::str::contains("No valid files found"));
}

#[test]
fn test_stdin_expand_parent_gitignore() {
    // Test that parent directory's .gitignore affects child directories
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // Create parent directory with .gitignore
    let parent = root.join("parent");
    fs::create_dir(&parent).unwrap();

    // Create .gitignore in parent that ignores "ignored_subdir"
    fs::write(parent.join(".gitignore"), "ignored_subdir/\n*.tmp\n").unwrap();

    // Create child directory
    let child = parent.join("child");
    fs::create_dir(&child).unwrap();

    // Create files in child
    fs::write(child.join("file1.txt"), "content1").unwrap();
    fs::write(child.join("file2.tmp"), "temp file").unwrap(); // Should be ignored by parent's .gitignore

    // Create ignored subdirectory in child
    let ignored_sub = child.join("ignored_subdir");
    fs::create_dir(&ignored_sub).unwrap();
    fs::write(ignored_sub.join("ignored.txt"), "ignored content").unwrap();

    // Test expanding child directory - should respect parent's .gitignore
    let assert = assert_cmd::Command::cargo_bin("tree2md")
        .expect("bin")
        .arg(&root)
        .arg("--stdin")
        .arg("--expand-dirs")
        .write_stdin(format!("{}\n", child.display()))
        .assert();

    let output = assert.success().get_output().clone();
    let out = String::from_utf8_lossy(&output.stdout).to_string();

    // Should include file1.txt
    assert!(out.contains("file1.txt"), "Should include file1.txt");

    // Should NOT include file2.tmp (ignored by parent's *.tmp pattern)
    assert!(!out.contains("file2.tmp"), "Should not include file2.tmp");

    // Should NOT include ignored_subdir (ignored by parent's ignored_subdir/ pattern)
    assert!(
        !out.contains("ignored_subdir"),
        "Should not include ignored_subdir"
    );
    assert!(
        !out.contains("ignored.txt"),
        "Should not include files in ignored_subdir"
    );
}

#[test]
fn test_sample_empty_file_handling() {
    let (_tmp, root) = setup_sample_dir();

    let (output, _, success) = run_tree2md([p(&root), "--contents".into()]);
    assert!(success);

    assert!(output.contains("### empty.txt"));
    if let Some(empty_idx) = output.find("### empty.txt") {
        let after_empty = &output[empty_idx..];
        assert!(after_empty.contains("```"));
    }
}

#[test]
fn test_sample_no_newline_file() {
    let (_tmp, root) = setup_sample_dir();

    let (output, _, success) = run_tree2md([p(&root), "--contents".into()]);
    assert!(success);

    if output.contains("### no_newline.txt") {
        let no_newline_idx = output.find("### no_newline.txt").unwrap();
        let after_no_newline = &output[no_newline_idx..];
        assert!(after_no_newline.contains("```"));
    }
}

#[test]
fn test_nonexistent_directory() {
    let bogus = PathBuf::from("nonexistent_directory_that_does_not_exist");
    let (_stdout, stderr, success) = run_tree2md([p(&bogus)]);
    assert!(!success);
    assert!(!stderr.is_empty());
}

#[test]
fn test_file_instead_of_directory() {
    let (_tmp, root) = setup_sample_dir();
    let hello = root.join("hello.py");

    // When pointing to a file directly, it should work
    let (output, _, success) = run_tree2md([p(&hello)]);
    assert!(success);
    assert!(output.contains("## File Structure"));
}

#[test]
fn test_empty_extension_list() {
    let (_tmp, root) = setup_sample_dir();

    let (output, _, success) = run_tree2md([p(&root), "--include-ext".into(), "".into()]);
    assert!(success);
    assert!(!output.contains("hello.py"));
    assert!(!output.contains("bar.go"));
}

#[test]
fn test_invalid_max_lines() {
    let (_tmp, root) = setup_sample_dir();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--contents".into(),
        "--max-lines".into(),
        "0".into(),
    ]);
    assert!(success);
    if output.contains("### hello.py") {
        let hello_idx = output.find("### hello.py").unwrap();
        let end = (hello_idx + 200).min(output.len());
        let after_hello = &output[hello_idx..end];
        assert!(after_hello.contains("Content truncated"));
    }
}

#[test]
fn test_multiple_strip_prefix() {
    let (_tmp, root) = setup_sample_dir();
    let foo = root.join("foo");

    let (_output, _stderr, success) = run_tree2md([
        p(&root),
        "--strip-prefix".into(),
        p(&root),
        "--strip-prefix".into(),
        p(&foo),
    ]);
    assert!(success);
}

#[test]
fn test_permission_denied() {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    let tmp = TempDir::new().unwrap();
    let root = tmp.path().to_path_buf();
    let locked = root.join("locked_dir");
    fs::create_dir_all(&locked).unwrap();
    fs::write(locked.join("file.txt"), "content").ok();

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&locked).unwrap().permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&locked, perms).ok();
    }

    let (_out, stderr, success) = run_tree2md([p(&locked)]);

    // restore to cleanup
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&locked).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&locked, perms).ok();
    }

    if !success {
        assert!(
            !stderr.is_empty(),
            "Should have error message for permission denied"
        );
    }
}

// --------------------------------------------------------------------------------
// Hidden files handling tests
// --------------------------------------------------------------------------------

#[test]
fn test_hidden_files_shown_by_default() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(root.join(".env"), "X=1").unwrap();
    fs::write(root.join(".gitignore"), "target/").unwrap();
    fs::write(root.join("visible.txt"), "visible").unwrap();

    let (out, _, ok) = run_tree2md([p(&root)]);
    assert!(ok);
    assert!(
        out.contains(".env"),
        "Hidden files should be shown by default"
    );
    assert!(
        out.contains(".gitignore"),
        "Hidden files should be shown by default"
    );
    assert!(out.contains("visible.txt"));
}

#[test]
fn test_exclude_hidden_flag() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::write(root.join(".env"), "X=1").unwrap();
    fs::write(root.join(".gitignore"), "target/").unwrap();
    fs::write(root.join("visible.txt"), "visible").unwrap();
    fs::create_dir(root.join(".hidden_dir")).unwrap();
    fs::write(root.join(".hidden_dir/file.txt"), "hidden").unwrap();

    let (out, _, ok) = run_tree2md([p(&root), "--exclude-hidden".into()]);
    assert!(ok);
    assert!(
        !out.contains(".env"),
        "Hidden files should be excluded with --exclude-hidden"
    );
    assert!(
        !out.contains(".gitignore"),
        "Hidden files should be excluded with --exclude-hidden"
    );
    assert!(
        !out.contains(".hidden_dir"),
        "Hidden directories should be excluded with --exclude-hidden"
    );
    assert!(
        out.contains("visible.txt"),
        "Visible files should still be shown"
    );
}

#[test]
fn test_stdin_expand_dirs_exclude_hidden() {
    use assert_cmd::Command;
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    fs::create_dir(root.join(".hidden_dir")).unwrap();
    fs::write(root.join(".hidden_dir/file.txt"), "x").unwrap();
    fs::write(root.join(".env"), "SECRET=1").unwrap();
    fs::write(root.join("visible.txt"), "visible").unwrap();

    let assert = Command::cargo_bin("tree2md")
        .expect("bin")
        .args([
            p(&root),
            "--stdin".into(),
            "--expand-dirs".into(),
            "--exclude-hidden".into(),
        ])
        .write_stdin(".\n")
        .assert();

    let output = assert.success().get_output().clone();
    let out = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        !out.contains(".hidden_dir"),
        "Hidden dirs should be excluded in stdin expand mode"
    );
    assert!(
        !out.contains(".env"),
        "Hidden files should be excluded in stdin expand mode"
    );
    assert!(
        out.contains("visible.txt"),
        "Visible files should be included in stdin expand mode"
    );
}

#[test]
fn test_git_dir_is_always_excluded() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();

    // Create .git directory with some files
    fs::create_dir(root.join(".git")).unwrap();
    fs::write(root.join(".git/config"), "[core]\nbare = false").unwrap();
    fs::write(root.join(".git/HEAD"), "ref: refs/heads/main").unwrap();
    fs::create_dir(root.join(".git/objects")).unwrap();

    // Create regular files
    fs::write(root.join("main.rs"), "fn main() {}").unwrap();
    fs::write(root.join(".env"), "SECRET=123").unwrap();

    // Test 1: Default behavior (shows hidden files but not .git)
    let (out, _, ok) = run_tree2md([p(&root)]);
    assert!(ok);
    assert!(out.contains("main.rs"), "Regular files should be shown");
    assert!(
        out.contains(".env"),
        "Hidden files should be shown by default"
    );
    assert!(
        !out.contains(".git"),
        ".git directory should always be excluded"
    );
    assert!(!out.contains("config"), ".git contents should not be shown");
    assert!(!out.contains("HEAD"), ".git contents should not be shown");

    // Test 2: Even with --no-gitignore, .git should still be excluded
    let (out, _, ok) = run_tree2md([p(&root), "--no-gitignore".into()]);
    assert!(ok);
    assert!(out.contains("main.rs"));
    assert!(out.contains(".env"));
    assert!(
        !out.contains(".git"),
        ".git should be excluded even with --no-gitignore"
    );

    // Test 3: stdin with expand-dirs should also exclude .git
    use assert_cmd::Command;
    let assert = Command::cargo_bin("tree2md")
        .expect("bin")
        .args([p(&root), "--stdin".into(), "--expand-dirs".into()])
        .write_stdin(".\n")
        .assert();

    let output = assert.success().get_output().clone();
    let out = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(out.contains("main.rs"));
    assert!(out.contains(".env"));
    assert!(
        !out.contains(".git"),
        ".git should be excluded in stdin expand mode"
    );
}
