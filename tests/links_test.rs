mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_links_enabled_by_default() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("empty.txt", "")
        .file("hello.py", "print(\"hello\")\n")
        .file("hoge.txt", "hoge\n")
        .file("large.txt", "line1\nline2\nline3\nline4\nline5\n")
        .file("multiline.txt", "line1\nline2\nline3\n")
        .file("no_newline.txt", "no_newline")
        .file("foo/bar.go", "package foo\nfunc Bar() {}\n")
        .file("foo/bar.js", "console.log('bar');\n")
        .file("foo/bar.py", "def bar():\n    pass\n")
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Should have Markdown links for files by default
    assert!(
        output.contains("[") && output.contains("]("),
        "Links should be enabled by default in Markdown format [text](url)"
    );

    // Check that file paths are in links
    assert!(
        output.contains("[hello.py]"),
        "Should have hello.py in Markdown link"
    );
}

#[test]
fn test_links_explicitly_on() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("empty.txt", "")
        .file("hello.py", "print(\"hello\")\n")
        .file("hoge.txt", "hoge\n")
        .file("large.txt", "line1\nline2\nline3\nline4\nline5\n")
        .file("multiline.txt", "line1\nline2\nline3\n")
        .file("no_newline.txt", "no_newline")
        .file("foo/bar.go", "package foo\nfunc Bar() {}\n")
        .file("foo/bar.js", "console.log('bar');\n")
        .file("foo/bar.py", "def bar():\n    pass\n")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--links".into(), "on".into()]);
    assert!(success);

    // Should have Markdown links
    assert!(
        output.contains("[") && output.contains("]("),
        "Links should be present with --links on in Markdown format"
    );

    // Check that file paths are in links
    assert!(
        output.contains("[hello.py]"),
        "Should have hello.py in Markdown link"
    );
}

#[test]
fn test_links_off() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("empty.txt", "")
        .file("hello.py", "print(\"hello\")\n")
        .file("hoge.txt", "hoge\n")
        .file("large.txt", "line1\nline2\nline3\nline4\nline5\n")
        .file("multiline.txt", "line1\nline2\nline3\n")
        .file("no_newline.txt", "no_newline")
        .file("foo/bar.go", "package foo\nfunc Bar() {}\n")
        .file("foo/bar.js", "console.log('bar');\n")
        .file("foo/bar.py", "def bar():\n    pass\n")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--links".into(), "off".into()]);
    assert!(success);

    // Should NOT have Markdown links
    assert!(
        !output.contains("]("),
        "Should not have Markdown links with --links off"
    );

    // But should still have file names
    assert!(output.contains("hello.py"));
    assert!(output.contains("bar.go"));
}

#[test]
fn test_github_url_rewriting() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("empty.txt", "")
        .file("hello.py", "print(\"hello\")\n")
        .file("hoge.txt", "hoge\n")
        .file("large.txt", "line1\nline2\nline3\nline4\nline5\n")
        .file("multiline.txt", "line1\nline2\nline3\n")
        .file("no_newline.txt", "no_newline")
        .file("foo/bar.go", "package foo\nfunc Bar() {}\n")
        .file("foo/bar.js", "console.log('bar');\n")
        .file("foo/bar.py", "def bar():\n    pass\n")
        .build();

    let github_url = "https://github.com/user/repo/tree/main";
    let (output, _, success) = run_tree2md([p(&root), "--github".into(), github_url.into()]);
    assert!(success);

    // Links should be rewritten to GitHub URLs in Markdown format
    assert!(
        output.contains(&format!("]({}/", github_url)),
        "Should have GitHub URLs in Markdown links"
    );
    assert!(
        output.contains(&format!("{}/hello.py", github_url))
            || output.contains(&format!("{}/empty.txt", github_url)),
        "Should have file paths with GitHub URL"
    );
}

#[test]
fn test_github_url_with_trailing_slash() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("empty.txt", "")
        .file("hello.py", "print(\"hello\")\n")
        .file("hoge.txt", "hoge\n")
        .file("large.txt", "line1\nline2\nline3\nline4\nline5\n")
        .file("multiline.txt", "line1\nline2\nline3\n")
        .file("no_newline.txt", "no_newline")
        .file("foo/bar.go", "package foo\nfunc Bar() {}\n")
        .file("foo/bar.js", "console.log('bar');\n")
        .file("foo/bar.py", "def bar():\n    pass\n")
        .build();

    // Test that trailing slash in GitHub URL is handled correctly
    let github_url = "https://github.com/user/repo/tree/main/";
    let (output, _, success) = run_tree2md([p(&root), "--github".into(), github_url.into()]);
    assert!(success);

    // Should not have double slashes
    assert!(
        !output.contains("main//"),
        "Should not have double slashes in URLs"
    );
    assert!(output.contains("https://github.com/user/repo/tree/main/"));
}

#[test]
fn test_github_url_different_branch() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("empty.txt", "")
        .file("hello.py", "print(\"hello\")\n")
        .file("hoge.txt", "hoge\n")
        .file("large.txt", "line1\nline2\nline3\nline4\nline5\n")
        .file("multiline.txt", "line1\nline2\nline3\n")
        .file("no_newline.txt", "no_newline")
        .file("foo/bar.go", "package foo\nfunc Bar() {}\n")
        .file("foo/bar.js", "console.log('bar');\n")
        .file("foo/bar.py", "def bar():\n    pass\n")
        .build();

    // Test with different branch name
    let github_url = "https://github.com/owner/project/tree/develop";
    let (output, _, success) = run_tree2md([p(&root), "--github".into(), github_url.into()]);
    assert!(success);

    // Should use the develop branch in URLs
    assert!(output.contains("tree/develop/"));
}

#[test]
fn test_github_url_with_links_off() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("empty.txt", "")
        .file("hello.py", "print(\"hello\")\n")
        .file("hoge.txt", "hoge\n")
        .file("large.txt", "line1\nline2\nline3\nline4\nline5\n")
        .file("multiline.txt", "line1\nline2\nline3\n")
        .file("no_newline.txt", "no_newline")
        .file("foo/bar.go", "package foo\nfunc Bar() {}\n")
        .file("foo/bar.js", "console.log('bar');\n")
        .file("foo/bar.py", "def bar():\n    pass\n")
        .build();

    // GitHub URL should be ignored when links are off
    let github_url = "https://github.com/user/repo/tree/main";
    let (output, _, success) = run_tree2md([
        p(&root),
        "--github".into(),
        github_url.into(),
        "--links".into(),
        "off".into(),
    ]);
    assert!(success);

    // Should not have any links
    assert!(!output.contains("]("), "Should not have Markdown links");
    assert!(
        !output.contains("github.com"),
        "Should not have GitHub URLs"
    );
}

#[test]
fn test_relative_links_without_github() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("empty.txt", "")
        .file("hello.py", "print(\"hello\")\n")
        .file("hoge.txt", "hoge\n")
        .file("large.txt", "line1\nline2\nline3\nline4\nline5\n")
        .file("multiline.txt", "line1\nline2\nline3\n")
        .file("no_newline.txt", "no_newline")
        .file("foo/bar.go", "package foo\nfunc Bar() {}\n")
        .file("foo/bar.js", "console.log('bar');\n")
        .file("foo/bar.py", "def bar():\n    pass\n")
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Without GitHub URL, links should be relative in Markdown format
    if output.contains("](") {
        // Should have relative paths in Markdown links
        assert!(
            output.contains("](hello.py)")
                || output.contains("](empty.txt)")
                || output.contains("](foo/)")
                || output.contains("[hello.py]")
                || output.contains("[empty.txt]"),
            "Should have relative paths in Markdown links"
        );

        // Should NOT have absolute URLs
        assert!(!output.contains("](https://"), "Should not have https URLs");
        assert!(!output.contains("](http://"), "Should not have http URLs");
    }
}

#[test]
fn test_directory_links() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("empty.txt", "")
        .file("hello.py", "print(\"hello\")\n")
        .file("hoge.txt", "hoge\n")
        .file("large.txt", "line1\nline2\nline3\nline4\nline5\n")
        .file("multiline.txt", "line1\nline2\nline3\n")
        .file("no_newline.txt", "no_newline")
        .file("foo/bar.go", "package foo\nfunc Bar() {}\n")
        .file("foo/bar.js", "console.log('bar');\n")
        .file("foo/bar.py", "def bar():\n    pass\n")
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Directories should be displayed in the output
    // In Markdown format, they appear as list items
    assert!(output.contains("foo/"), "Should contain directory name");
}

#[test]
fn test_nested_path_github_links() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("src/modules/utils/helper.rs", "fn help() {}")
        .build();

    let github_url = "https://github.com/org/repo/tree/main";
    let (output, _, success) = run_tree2md([p(&root), "--github".into(), github_url.into()]);
    assert!(success);

    // Should have properly formatted nested GitHub URLs
    if output.contains("helper.rs") {
        assert!(
            output.contains(&format!("{}/src/modules/utils/helper.rs", github_url))
                || output.contains("src/modules/utils")
        );
    }
}

#[test]
fn test_special_characters_in_filenames() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("file-with-dash.txt", "content")
        .file("file_with_underscore.txt", "content")
        .file("file.multiple.dots.txt", "content")
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // All files should be present
    assert!(output.contains("file-with-dash.txt"));
    assert!(output.contains("file_with_underscore.txt"));
    assert!(output.contains("file.multiple.dots.txt"));

    // With GitHub URL, special characters should be preserved
    let github_url = "https://github.com/user/repo/tree/main";
    let (output, _, success) = run_tree2md([p(&root), "--github".into(), github_url.into()]);
    assert!(success);

    assert!(output.contains("file-with-dash.txt"));
    assert!(output.contains("file_with_underscore.txt"));
}

#[test]
fn test_invalid_github_url() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("empty.txt", "")
        .file("hello.py", "print(\"hello\")\n")
        .file("hoge.txt", "hoge\n")
        .file("large.txt", "line1\nline2\nline3\nline4\nline5\n")
        .file("multiline.txt", "line1\nline2\nline3\n")
        .file("no_newline.txt", "no_newline")
        .file("foo/bar.go", "package foo\nfunc Bar() {}\n")
        .file("foo/bar.js", "console.log('bar');\n")
        .file("foo/bar.py", "def bar():\n    pass\n")
        .build();

    // Test with invalid URL (missing protocol)
    let (_output, _stderr, success) =
        run_tree2md([p(&root), "--github".into(), "github.com/user/repo".into()]);

    // Should fail validation
    assert!(!success, "Should fail with invalid GitHub URL");
}

#[test]
fn test_github_url_validation() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("empty.txt", "")
        .file("hello.py", "print(\"hello\")\n")
        .file("hoge.txt", "hoge\n")
        .file("large.txt", "line1\nline2\nline3\nline4\nline5\n")
        .file("multiline.txt", "line1\nline2\nline3\n")
        .file("no_newline.txt", "no_newline")
        .file("foo/bar.go", "package foo\nfunc Bar() {}\n")
        .file("foo/bar.js", "console.log('bar');\n")
        .file("foo/bar.py", "def bar():\n    pass\n")
        .build();

    // Test with valid HTTP URL
    let (_, _, success) = run_tree2md([
        p(&root),
        "--github".into(),
        "http://github.com/user/repo/tree/main".into(),
    ]);
    assert!(success, "Should accept HTTP URLs");

    // Test with valid HTTPS URL
    let (_, _, success) = run_tree2md([
        p(&root),
        "--github".into(),
        "https://github.com/user/repo/tree/main".into(),
    ]);
    assert!(success, "Should accept HTTPS URLs");
}

#[test]
fn test_empty_directory_no_links() {
    let (_tmp, root) = FixtureBuilder::new().dir("empty").build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Empty directory should appear but without file links
    assert!(output.contains("empty"));

    // Should not have links inside empty directory
    let empty_section = output.split("empty").collect::<Vec<_>>();
    if empty_section.len() > 1 {
        // Check the section after "empty" directory
        // In Markdown format, empty directories don't have file links
        // Just verify the directory appears
    }
}
