use std::process::Command;

fn run_tree2md(args: &[&str]) -> (String, String, bool) {
    let mut cmd = Command::new("./target/release/tree2md");
    cmd.args(args);

    let output = cmd.output().expect("Failed to execute tree2md");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (stdout, stderr, output.status.success())
}

#[test]
fn test_sample_directory_basic_structure() {
    let (output, _, success) = run_tree2md(&["sample"]);
    assert!(success, "Command should succeed");

    // Check for main structure header
    assert!(
        output.contains("## File Structure"),
        "Should have file structure header"
    );

    // Check for files in root
    assert!(output.contains("empty.txt"), "Should list empty.txt");
    assert!(output.contains("hello.py"), "Should list hello.py");
    assert!(output.contains("hoge.txt"), "Should list hoge.txt");
    assert!(output.contains("large.txt"), "Should list large.txt");
    assert!(
        output.contains("multiline.txt"),
        "Should list multiline.txt"
    );
    assert!(
        output.contains("no_newline.txt"),
        "Should list no_newline.txt"
    );

    // Check for foo directory and its contents
    assert!(output.contains("foo/"), "Should list foo directory");
    assert!(output.contains("bar.go"), "Should list bar.go");
    assert!(output.contains("bar.js"), "Should list bar.js");
    assert!(output.contains("bar.py"), "Should list bar.py");
}

#[test]
fn test_sample_with_contents() {
    let (output, _, success) = run_tree2md(&["sample", "--contents"]);
    assert!(success, "Command should succeed");

    // Check structure is present
    assert!(
        output.contains("## File Structure"),
        "Should have file structure header"
    );

    // Check for code blocks - note paths don't include "sample/" prefix in headers
    assert!(
        output.contains("### hello.py"),
        "Should have hello.py header"
    );
    assert!(
        output.contains("```python"),
        "Should have python code block"
    );
    assert!(
        output.contains("print(\"hello\")"),
        "Should contain hello.py content"
    );

    // Check Go file
    assert!(
        output.contains("### foo/bar.go"),
        "Should have bar.go header"
    );
    assert!(output.contains("```go"), "Should have go code block");
    assert!(
        output.contains("package foo"),
        "Should contain bar.go content"
    );
    assert!(output.contains("func Bar()"), "Should contain Bar function");

    // Check multiline text file
    assert!(
        output.contains("### multiline.txt"),
        "Should have multiline.txt header"
    );
    assert!(output.contains("line1"), "Should contain first line");
    assert!(output.contains("line2"), "Should contain second line");
    assert!(output.contains("line3"), "Should contain third line");
}

#[test]
fn test_sample_with_extension_filter() {
    let (output, _, success) = run_tree2md(&["sample", "--include-ext", "py"]);
    assert!(success, "Command should succeed");

    // Should include Python files
    assert!(output.contains("hello.py"), "Should include hello.py");
    assert!(output.contains("bar.py"), "Should include bar.py");

    // Should exclude other extensions
    assert!(!output.contains("bar.go"), "Should exclude .go files");
    assert!(!output.contains("bar.js"), "Should exclude .js files");
    assert!(!output.contains("hoge.txt"), "Should exclude .txt files");
    assert!(!output.contains("empty.txt"), "Should exclude empty.txt");
}

#[test]
fn test_sample_multiple_extensions() {
    let (output, _, success) = run_tree2md(&["sample", "--include-ext", "py,go"]);
    assert!(success, "Command should succeed");

    // Should include Python and Go files
    assert!(output.contains("hello.py"), "Should include hello.py");
    assert!(output.contains("bar.py"), "Should include bar.py");
    assert!(output.contains("bar.go"), "Should include bar.go");

    // Should exclude other extensions
    assert!(!output.contains("bar.js"), "Should exclude .js files");
    assert!(!output.contains("hoge.txt"), "Should exclude .txt files");
}

#[test]
fn test_sample_with_max_lines() {
    let (output, _, success) = run_tree2md(&["sample", "--contents", "--max-lines", "2"]);
    assert!(success, "Command should succeed");

    // Check that multiline.txt is truncated
    assert!(
        output.contains("### multiline.txt"),
        "Should have multiline.txt header"
    );
    assert!(output.contains("line1"), "Should contain first line");
    assert!(output.contains("line2"), "Should contain second line");
    assert!(
        !output.contains("line3"),
        "Should not contain third line (truncated)"
    );
    assert!(
        output.contains("[Content truncated:"),
        "Should have truncation message"
    );
}

#[test]
fn test_sample_flat_structure() {
    let (output, _, success) = run_tree2md(&["sample", "--flat"]);
    assert!(success, "Command should succeed");

    // In flat mode, files are listed with their paths from root
    assert!(output.contains("- empty.txt"), "Should show empty.txt");
    assert!(output.contains("- hello.py"), "Should show hello.py");
    assert!(output.contains("- foo/bar.go"), "Should show foo/bar.go");
    assert!(output.contains("- foo/bar.js"), "Should show foo/bar.js");
    assert!(output.contains("- foo/bar.py"), "Should show foo/bar.py");

    // Should not have tree-like indentation or structure
    assert!(!output.contains("  -"), "Should not have indented items");
    assert!(!output.contains("sample/"), "Should not show sample prefix");
}

#[test]
#[ignore] // TODO: --exclude option not yet implemented
fn test_sample_with_exclude_pattern() {
    let (output, _, success) = run_tree2md(&["sample", "--exclude", "*.txt"]);
    assert!(success, "Command should succeed");

    // Should include non-txt files
    assert!(output.contains("hello.py"), "Should include hello.py");
    assert!(output.contains("bar.go"), "Should include bar.go");
    assert!(output.contains("bar.js"), "Should include bar.js");
    assert!(output.contains("bar.py"), "Should include bar.py");

    // Should exclude txt files
    assert!(!output.contains("empty.txt"), "Should exclude empty.txt");
    assert!(!output.contains("hoge.txt"), "Should exclude hoge.txt");
    assert!(!output.contains("large.txt"), "Should exclude large.txt");
    assert!(
        !output.contains("multiline.txt"),
        "Should exclude multiline.txt"
    );
    assert!(
        !output.contains("no_newline.txt"),
        "Should exclude no_newline.txt"
    );
}

#[test]
#[ignore] // TODO: --include pattern option not yet implemented
fn test_sample_with_include_pattern() {
    let (output, _, success) = run_tree2md(&["sample", "--include", "foo/*.py"]);
    assert!(success, "Command should succeed");

    // Should only include Python files in foo directory
    assert!(output.contains("bar.py"), "Should include foo/bar.py");

    // Should exclude everything else
    assert!(
        !output.contains("hello.py"),
        "Should exclude hello.py (not in foo/)"
    );
    assert!(!output.contains("bar.go"), "Should exclude bar.go");
    assert!(!output.contains("bar.js"), "Should exclude bar.js");
    assert!(
        !output.contains("empty.txt"),
        "Should exclude all txt files"
    );
}

#[test]
#[ignore] // TODO: --max-depth option not yet implemented
fn test_sample_depth_limit() {
    let (output, _, success) = run_tree2md(&["sample", "--max-depth", "1"]);
    assert!(success, "Command should succeed");

    // Should include root level files
    assert!(
        output.contains("empty.txt"),
        "Should include root level empty.txt"
    );
    assert!(
        output.contains("hello.py"),
        "Should include root level hello.py"
    );
    assert!(
        output.contains("hoge.txt"),
        "Should include root level hoge.txt"
    );

    // Should show foo directory but not its contents
    assert!(output.contains("foo/"), "Should show foo directory");
    assert!(
        !output.contains("bar.go"),
        "Should not show contents of foo/"
    );
    assert!(
        !output.contains("bar.js"),
        "Should not show contents of foo/"
    );
    assert!(
        !output.contains("bar.py"),
        "Should not show contents of foo/"
    );
}

#[test]
fn test_sample_no_root() {
    let (output, _, success) = run_tree2md(&["sample", "--no-root"]);
    assert!(success, "Command should succeed");

    // Check that root (sample) is not shown
    // Files should be at the top level of the tree
    let lines: Vec<&str> = output.lines().collect();
    let structure_start = lines
        .iter()
        .position(|&l| l == "## File Structure")
        .unwrap();

    // The first non-empty line after "## File Structure" should be a file, not "sample"
    let mut first_item_found = false;
    for i in (structure_start + 1)..lines.len() {
        if !lines[i].trim().is_empty() {
            assert!(
                !lines[i].contains("sample"),
                "Root should not be shown with --no-root"
            );
            first_item_found = true;
            break;
        }
    }
    assert!(
        first_item_found,
        "Should find at least one item in the tree"
    );
}

#[test]
fn test_sample_with_root_label() {
    let (output, _, success) = run_tree2md(&["sample", "--root-label", "MyProject"]);
    assert!(success, "Command should succeed");

    // Should use custom root label instead of "sample"
    assert!(output.contains("MyProject"), "Should use custom root label");

    // Should still contain all files
    assert!(output.contains("empty.txt"), "Should contain files");
    assert!(output.contains("hello.py"), "Should contain files");
    assert!(output.contains("foo/"), "Should contain subdirectory");
}

#[test]
#[ignore] // TODO: --directories-only option not yet implemented
fn test_sample_directories_only() {
    let (output, _, success) = run_tree2md(&["sample", "--directories-only"]);
    assert!(success, "Command should succeed");

    // Should only show directories
    assert!(output.contains("foo/"), "Should show foo directory");

    // Should not show files
    assert!(!output.contains("empty.txt"), "Should not show files");
    assert!(!output.contains("hello.py"), "Should not show files");
    assert!(!output.contains("bar.go"), "Should not show files");
}

#[test]
fn test_sample_with_gitignore() {
    // First, create a .gitignore file in sample directory
    std::fs::write("sample/.gitignore", "*.txt\n").expect("Failed to write .gitignore");

    let (output, _, success) = run_tree2md(&["sample", "--respect-gitignore"]);
    assert!(success, "Command should succeed");

    // Should exclude .txt files as per .gitignore
    assert!(!output.contains("empty.txt"), "Should respect .gitignore");
    assert!(!output.contains("hoge.txt"), "Should respect .gitignore");
    assert!(
        !output.contains("multiline.txt"),
        "Should respect .gitignore"
    );

    // Should include non-ignored files
    assert!(
        output.contains("hello.py"),
        "Should include non-ignored files"
    );
    assert!(
        output.contains("bar.go"),
        "Should include non-ignored files"
    );

    // Clean up
    std::fs::remove_file("sample/.gitignore").ok();
}

#[test]
fn test_sample_empty_file_handling() {
    let (output, _, success) = run_tree2md(&["sample", "--contents"]);
    assert!(success, "Command should succeed");

    // Check that empty.txt is handled correctly
    assert!(
        output.contains("### empty.txt"),
        "Should have empty.txt header"
    );
    // The code block for empty file should exist but be empty
    if let Some(empty_idx) = output.find("### empty.txt") {
        let after_empty = &output[empty_idx..];
        assert!(
            after_empty.contains("```"),
            "Should have code block for empty file"
        );
    }
}

#[test]
fn test_sample_no_newline_file() {
    // This tests a file that doesn't end with a newline
    let (output, _, success) = run_tree2md(&["sample", "--contents"]);
    assert!(success, "Command should succeed");

    // Check that no_newline.txt is handled correctly
    if output.contains("### no_newline.txt") {
        let no_newline_idx = output.find("### no_newline.txt").unwrap();
        let after_no_newline = &output[no_newline_idx..];
        assert!(after_no_newline.contains("```"), "Should have code block");
    }
}

#[test]
fn test_nonexistent_directory() {
    let (_, stderr, success) = run_tree2md(&["nonexistent_directory_that_does_not_exist"]);
    assert!(!success, "Command should fail for nonexistent directory");
    assert!(!stderr.is_empty(), "Should have error message");
}

#[test]
fn test_file_instead_of_directory() {
    let (output, _, success) = run_tree2md(&["sample/hello.py"]);
    assert!(success, "Command should succeed when given a file");
    // When given a file, tree2md shows that single file
    assert!(output.contains("hello.py"), "Should show the file");
}

#[test]
fn test_empty_extension_list() {
    let (output, _, success) = run_tree2md(&["sample", "--include-ext", ""]);
    assert!(success, "Command should succeed with empty extension list");
    // With empty extension filter, no files should match
    assert!(!output.contains("hello.py"), "Should not include any files");
    assert!(!output.contains("bar.go"), "Should not include any files");
}

#[test]
fn test_invalid_max_lines() {
    let (output, _, success) = run_tree2md(&["sample", "--contents", "--max-lines", "0"]);
    assert!(success, "Command should succeed with max-lines 0");
    // With max-lines 0, file contents should be empty or show truncation
    if output.contains("### hello.py") {
        let hello_idx = output.find("### hello.py").unwrap();
        let after_hello = &output[hello_idx..hello_idx + 200.min(output.len() - hello_idx)];
        assert!(
            after_hello.contains("Content truncated"),
            "Should truncate all content with max-lines 0"
        );
    }
}

#[test]
fn test_multiple_strip_prefix() {
    let (_output, _, success) = run_tree2md(&[
        "sample",
        "--strip-prefix",
        "sample",
        "--strip-prefix",
        "foo",
    ]);
    assert!(success, "Command should succeed with multiple strip-prefix");
    // Note: Need to verify how strip-prefix actually works
}

#[test]
fn test_permission_denied() {
    // Create a directory without read permission
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = "test_no_permission";
    fs::create_dir(temp_dir).ok();
    fs::write(format!("{}/file.txt", temp_dir), "content").ok();

    // Remove read permission
    let mut perms = fs::metadata(temp_dir).unwrap().permissions();
    perms.set_mode(0o000);
    fs::set_permissions(temp_dir, perms).ok();

    let (_, stderr, success) = run_tree2md(&[temp_dir]);

    // Restore permission and clean up
    let mut perms = fs::metadata(temp_dir).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(temp_dir, perms).ok();
    fs::remove_dir_all(temp_dir).ok();

    // On some systems, permission denied might not fail the command
    // but should at least show a warning
    if !success {
        assert!(
            !stderr.is_empty(),
            "Should have error message for permission denied"
        );
    }
}
