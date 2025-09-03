mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_output_format_has_details_tags() {
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

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Check for HTML details tags (from OUTPUT_SAMPLE.md spec)
    // Note: sample_dir has a 'foo' directory, so should have details tags
    assert!(
        output.contains("<details"),
        "Output should contain <details> tags for folders in HTML output"
    );
    assert!(
        output.contains("</details>"),
        "Output should contain closing </details> tags"
    );
    assert!(
        output.contains("<summary>"),
        "Output should contain <summary> tags"
    );
}

#[test]
fn test_fold_mode_auto() {
    let builder = FixtureBuilder::new();
    let builder = (0..20).fold(builder, |b, i| {
        b.file(&format!("many/file{}.txt", i), "content")
    });
    let (_tmp, root) = builder.build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--output".into(),
        "html".into(),
        "--fold".into(),
        "auto".into(),
    ]);
    assert!(success);

    // With many files in HTML output, should have details tag
    assert!(
        output.contains("<details"),
        "Large directories should auto-fold in HTML output"
    );
}

#[test]
fn test_fold_mode_off() {
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

    let (output, _, success) = run_tree2md([p(&root), "--fold".into(), "off".into()]);
    assert!(success);

    // Should not have details tags
    assert!(
        !output.contains("<details>"),
        "Fold off should not have details tags"
    );
}

#[test]
fn test_stats_footer() {
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

    // Check for stats footer
    assert!(
        output.contains("**Totals**") || output.contains("**Stats**"),
        "Should have stats header"
    );
    assert!(
        output.contains("dirs") || output.contains("Dirs:"),
        "Should show directory count"
    );
    assert!(
        output.contains("files") || output.contains("Files:"),
        "Should show file count"
    );
    assert!(
        output.contains("By type") || output.contains("Top by ext"),
        "Should show file type breakdown"
    );
}

#[test]
fn test_no_stats_flag() {
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

    let (output, _, success) = run_tree2md([p(&root), "--no-stats".into()]);
    assert!(success);

    // Should not have stats
    assert!(
        !output.contains("**Stats**"),
        "No-stats flag should suppress stats"
    );
}

#[test]
fn test_safe_mode_default() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Project\n")
        .file("main.rs", "fn main() {}\n")
        .file(".env", "API_KEY=secret123\n")
        .file(".env.local", "LOCAL_KEY=local456\n")
        .dir(".ssh")
        .file(".ssh/id_rsa", "-----BEGIN RSA PRIVATE KEY-----\n")
        .file(".ssh/id_rsa.pub", "ssh-rsa AAAAB3...\n")
        .file(".ssh/config", "Host *\n")
        .file("server.pem", "-----BEGIN CERTIFICATE-----\n")
        .file("private.key", "-----BEGIN PRIVATE KEY-----\n")
        .file("cert.crt", "-----BEGIN CERTIFICATE-----\n")
        .file("target/debug/app", "binary")
        .file("node_modules/package/index.js", "module.exports")
        .file(".DS_Store", "\0")
        .file("Thumbs.db", "\0")
        .build();

    // Default should be safe mode
    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Should exclude sensitive files by default
    assert!(
        !output.contains(".env"),
        ".env should be excluded by default"
    );
    assert!(
        !output.contains("id_rsa"),
        "SSH keys should be excluded by default"
    );
    assert!(
        !output.contains("private.key"),
        "Private keys should be excluded by default"
    );
}

#[test]
fn test_deterministic_order() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("z_file.txt", "z")
        .file("a_file.txt", "a")
        .dir("b_dir")
        .dir("y_dir")
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Check order: directories first (alphabetical), then files (alphabetical)
    let lines: Vec<&str> = output.lines().collect();
    let mut found_items = vec![];

    for line in lines {
        if line.contains("b_dir") {
            found_items.push("b_dir");
        }
        if line.contains("y_dir") {
            found_items.push("y_dir");
        }
        if line.contains("a_file") {
            found_items.push("a_file");
        }
        if line.contains("z_file") {
            found_items.push("z_file");
        }
    }

    // Should be: dirs first (b_dir, y_dir), then files (a_file, z_file)
    assert_eq!(
        found_items,
        vec!["b_dir", "y_dir", "a_file", "z_file"],
        "Output should be deterministic: dirs first, then files, both alphabetical"
    );
}

#[test]
fn test_symlinks_skipped() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("real_file.txt", "real")
        .file("normal.txt", "normal")
        .build();

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(root.join("real_file.txt"), root.join("symlink.txt")).unwrap();
        symlink("/etc", root.join("symlink_dir")).unwrap();
    }

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    #[cfg(unix)]
    {
        assert!(
            !output.contains("symlink.txt"),
            "Symlinks should be skipped"
        );
        assert!(
            !output.contains("symlink_dir"),
            "Symlink directories should be skipped"
        );
    }
    assert!(
        output.contains("real_file.txt") || output.contains("[real_file.txt]"),
        "Real files should be included"
    );
}

#[test]
fn test_exit_code_success() {
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

    let (_, _, success) = run_tree2md([p(&root)]);
    assert!(success, "Should exit with 0 on success");
}

#[test]
fn test_exit_code_invalid_args() {
    // Invalid argument combination or invalid path
    let (_, _, success) = run_tree2md(vec!["--invalid-flag"]);
    assert!(!success, "Should exit with non-zero on invalid args");
}

#[test]
fn test_inject_flag_present() {
    // Just test that --inject is recognized (actual injection tested elsewhere)
    let (_tmp, root) = FixtureBuilder::new()
        .file("test.txt", "test")
        .file("README.md", "# Test\n")
        .build();
    let readme = root.join("README.md");

    // This should recognize the flag even if it doesn't actually inject in tests
    let (_, stderr, _) = run_tree2md([p(&root), "--inject".into(), p(&readme)]);

    // Should either work or give a specific error, not "unknown flag"
    assert!(
        !stderr.contains("unexpected argument '--inject'"),
        "--inject flag should be recognized"
    );
}

#[test]
fn test_dry_run_flag_present() {
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

    // Test that --dry-run is recognized
    let (_, stderr, _) = run_tree2md([p(&root), "--dry-run".into()]);

    assert!(
        !stderr.contains("unexpected argument '--dry-run'"),
        "--dry-run flag should be recognized"
    );
}

#[test]
fn test_tag_customization() {
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

    // Test custom tags are recognized
    let (_, stderr, _) = run_tree2md([
        p(&root),
        "--tag-start".into(),
        "<!-- custom:start -->".into(),
        "--tag-end".into(),
        "<!-- custom:end -->".into(),
    ]);

    assert!(
        !stderr.contains("unexpected argument '--tag-start'"),
        "Custom tag flags should be recognized"
    );
}

#[test]
fn test_stamp_date_format_flag() {
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

    let (_, stderr, _) = run_tree2md([p(&root), "--stamp-date-format".into(), "%Y/%m/%d".into()]);

    assert!(
        !stderr.contains("unexpected argument '--stamp-date-format'"),
        "--stamp-date-format flag should be recognized"
    );
}

#[test]
fn test_output_starts_with_ul() {
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

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Per OUTPUT_SAMPLE.md, HTML output should start with <ul>
    let trimmed = output.trim_start();
    assert!(
        trimmed.starts_with("<ul>"),
        "HTML output should start with <ul> tag per spec"
    );
}

#[test]
fn test_output_has_proper_nesting() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("root.txt", "root")
        .file("level1/file1.txt", "f1")
        .file("level1/level2/file2.txt", "f2")
        .file("level1/level2/level3/file3.txt", "f3")
        .file("dir1/file.txt", "test")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Should have proper HTML list nesting in HTML output
    assert!(
        output.contains("<ul>"),
        "HTML output should have unordered list"
    );
    assert!(output.contains("<li>"), "Should have list items");
    assert!(output.contains("</li>"), "Should close list items");
    assert!(output.contains("</ul>"), "Should close unordered list");
}
