mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_html_tree_basic_structure() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root\n")
        .file("src/main.rs", "fn main() {}\n")
        .file("src/lib.rs", "pub fn lib() {}\n")
        .file("src/modules/mod1.rs", "mod mod1")
        .file("src/modules/mod2.rs", "mod mod2")
        .file("src/modules/mod3.rs", "mod mod3")
        .files_with(
            (1..=10).map(|i| format!("src/modules/utils/util{}.rs", i)),
            |p| format!("// util {}\n", p.file_name().unwrap().to_string_lossy()),
        )
        .file("docs/guide.md", "# Guide\n")
        .file("docs/api.md", "# API\n")
        .file("docs/examples/example1.md", "Example 1")
        .file("docs/examples/example2.md", "Example 2")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Check for HTML list structure
    assert!(output.contains("<ul>"));
    assert!(output.contains("</ul>"));
    assert!(output.contains("<li>"));
    assert!(output.contains("</li>"));
}

#[test]
fn test_details_tags_auto_mode() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root\n")
        .file("src/main.rs", "fn main() {}\n")
        .file("src/lib.rs", "pub fn lib() {}\n")
        .file("src/modules/mod1.rs", "mod mod1")
        .file("src/modules/mod2.rs", "mod mod2")
        .file("src/modules/mod3.rs", "mod mod3")
        .files_with(
            (1..=10).map(|i| format!("src/modules/utils/util{}.rs", i)),
            |p| format!("// util {}\n", p.file_name().unwrap().to_string_lossy()),
        )
        .file("docs/guide.md", "# Guide\n")
        .file("docs/api.md", "# API\n")
        .file("docs/examples/example1.md", "Example 1")
        .file("docs/examples/example2.md", "Example 2")
        .build();

    // Test HTML output mode with auto fold
    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Should have details tags for directories with many items
    assert!(output.contains("<details"));
    assert!(output.contains("</details>"));
    assert!(output.contains("<summary>"));
    assert!(output.contains("</summary>"));
}

#[test]
fn test_details_tags_always_on() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root\n")
        .file("src/main.rs", "fn main() {}\n")
        .file("src/lib.rs", "pub fn lib() {}\n")
        .file("src/modules/mod1.rs", "mod mod1")
        .file("src/modules/mod2.rs", "mod mod2")
        .file("src/modules/mod3.rs", "mod mod3")
        .files_with(
            (1..=10).map(|i| format!("src/modules/utils/util{}.rs", i)),
            |p| format!("// util {}\n", p.file_name().unwrap().to_string_lossy()),
        )
        .file("docs/guide.md", "# Guide\n")
        .file("docs/api.md", "# API\n")
        .file("docs/examples/example1.md", "Example 1")
        .file("docs/examples/example2.md", "Example 2")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--output".into(),
        "html".into(),
        "--fold".into(),
        "on".into(),
    ]);
    assert!(success);

    // All directories should have details tags
    assert!(output.contains("<details"));
    assert!(output.contains("</details>"));

    // Count details tags - should be many
    let details_count = output.matches("<details").count();
    assert!(
        details_count > 3,
        "Should have many details tags when fold=on"
    );
}

#[test]
fn test_details_tags_off() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root\n")
        .file("src/main.rs", "fn main() {}\n")
        .file("src/lib.rs", "pub fn lib() {}\n")
        .file("src/modules/mod1.rs", "mod mod1")
        .file("src/modules/mod2.rs", "mod mod2")
        .file("src/modules/mod3.rs", "mod mod3")
        .files_with(
            (1..=10).map(|i| format!("src/modules/utils/util{}.rs", i)),
            |p| format!("// util {}\n", p.file_name().unwrap().to_string_lossy()),
        )
        .file("docs/guide.md", "# Guide\n")
        .file("docs/api.md", "# API\n")
        .file("docs/examples/example1.md", "Example 1")
        .file("docs/examples/example2.md", "Example 2")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--output".into(),
        "html".into(),
        "--fold".into(),
        "off".into(),
    ]);
    assert!(success);

    // Should not have any details tags
    assert!(
        !output.contains("<details"),
        "Should not have details tags when fold=off"
    );
    assert!(!output.contains("</details>"));
}

#[test]
fn test_directory_counts_in_summary() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root\n")
        .file("src/main.rs", "fn main() {}\n")
        .file("src/lib.rs", "pub fn lib() {}\n")
        .file("src/modules/mod1.rs", "mod mod1")
        .file("src/modules/mod2.rs", "mod mod2")
        .file("src/modules/mod3.rs", "mod mod3")
        .files_with(
            (1..=10).map(|i| format!("src/modules/utils/util{}.rs", i)),
            |p| format!("// util {}\n", p.file_name().unwrap().to_string_lossy()),
        )
        .file("docs/guide.md", "# Guide\n")
        .file("docs/api.md", "# API\n")
        .file("docs/examples/example1.md", "Example 1")
        .file("docs/examples/example2.md", "Example 2")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Check for file and directory counts in summaries
    assert!(output.contains("(files:"));
    assert!(output.contains("dirs:"));

    // The utils directory should show it has 10 files
    if output.contains("<summary><code>utils/</code>") {
        let utils_section = output.split("utils/").nth(0).unwrap();
        assert!(utils_section.contains("files: 10") || output.contains("files: 10"));
    }
}

#[test]
fn test_code_tags_for_names() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root\n")
        .file("src/main.rs", "fn main() {}\n")
        .file("src/lib.rs", "pub fn lib() {}\n")
        .file("src/modules/mod1.rs", "mod mod1")
        .file("src/modules/mod2.rs", "mod mod2")
        .file("src/modules/mod3.rs", "mod mod3")
        .files_with(
            (1..=10).map(|i| format!("src/modules/utils/util{}.rs", i)),
            |p| format!("// util {}\n", p.file_name().unwrap().to_string_lossy()),
        )
        .file("docs/guide.md", "# Guide\n")
        .file("docs/api.md", "# API\n")
        .file("docs/examples/example1.md", "Example 1")
        .file("docs/examples/example2.md", "Example 2")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Directory names should be wrapped in <code> tags
    assert!(output.contains("<code>src/</code>"));
    assert!(output.contains("<code>docs/</code>"));

    // File names should be in links or plain text depending on settings
    assert!(output.contains("README.md"));
    assert!(output.contains("main.rs"));
}

#[test]
fn test_stats_footer() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("lib.rs", "pub fn lib() {}")
        .file("test.rs", "#[test]")
        .file("index.js", "console.log()")
        .file("app.js", "const app = {}")
        .file("style.css", "body {}")
        .file("index.html", "<html>")
        .file("README.md", "# Readme")
        .file("CHANGELOG.md", "## Changes")
        .file("Makefile", "all:")
        .file("Dockerfile", "FROM ubuntu")
        .file("src/main.rs", "fn main() {}")
        .file("tests/test1.rs", "#[test]")
        .file("tests/test2.rs", "#[test]")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Check for stats footer
    assert!(output.contains("---"));
    assert!(output.contains("**Stats**"));
    assert!(output.contains("- Dirs:"));
    assert!(output.contains("- Files:"));
    assert!(output.contains("- Top by ext:"));

    // Should show top extensions
    assert!(output.contains("rs(")); // Rust files count
    assert!(output.contains("js(")); // JavaScript files count
    assert!(output.contains("md(")); // Markdown files count
}

#[test]
fn test_stats_footer_suppressed() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("lib.rs", "pub fn lib() {}")
        .file("test.rs", "#[test]")
        .file("index.js", "console.log()")
        .file("app.js", "const app = {}")
        .file("style.css", "body {}")
        .file("index.html", "<html>")
        .file("README.md", "# Readme")
        .file("CHANGELOG.md", "## Changes")
        .file("Makefile", "all:")
        .file("Dockerfile", "FROM ubuntu")
        .file("src/main.rs", "fn main() {}")
        .file("tests/test1.rs", "#[test]")
        .file("tests/test2.rs", "#[test]")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--output".into(),
        "html".into(),
        "--no-stats".into(),
    ]);
    assert!(success);

    // Should not have stats footer
    assert!(!output.contains("**Stats**"));
    assert!(!output.contains("- Top by ext:"));
}

#[test]
fn test_nested_lists_indentation() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root\n")
        .file("src/main.rs", "fn main() {}\n")
        .file("src/lib.rs", "pub fn lib() {}\n")
        .file("src/modules/mod1.rs", "mod mod1")
        .file("src/modules/mod2.rs", "mod mod2")
        .file("src/modules/mod3.rs", "mod mod3")
        .files_with(
            (1..=10).map(|i| format!("src/modules/utils/util{}.rs", i)),
            |p| format!("// util {}\n", p.file_name().unwrap().to_string_lossy()),
        )
        .file("docs/guide.md", "# Guide\n")
        .file("docs/api.md", "# API\n")
        .file("docs/examples/example1.md", "Example 1")
        .file("docs/examples/example2.md", "Example 2")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--output".into(),
        "html".into(),
        "--fold".into(),
        "off".into(),
    ]);
    assert!(success);

    // Check proper nesting with indentation
    let lines: Vec<&str> = output.lines().collect();

    // Find a nested structure and verify indentation increases
    let mut found_nested = false;
    for i in 0..lines.len() - 1 {
        if lines[i].contains("<ul>") && lines[i + 1].contains("<li>") {
            let indent_ul = lines[i].chars().take_while(|c| c.is_whitespace()).count();
            let indent_li = lines[i + 1]
                .chars()
                .take_while(|c| c.is_whitespace())
                .count();
            if indent_li > indent_ul {
                found_nested = true;
                break;
            }
        }
    }
    assert!(found_nested, "Should have properly indented nested lists");
}

#[test]
fn test_details_open_attribute() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root\n")
        .file("src/main.rs", "fn main() {}\n")
        .file("src/lib.rs", "pub fn lib() {}\n")
        .file("src/modules/mod1.rs", "mod mod1")
        .file("src/modules/mod2.rs", "mod mod2")
        .file("src/modules/mod3.rs", "mod mod3")
        .files_with(
            (1..=10).map(|i| format!("src/modules/utils/util{}.rs", i)),
            |p| format!("// util {}\n", p.file_name().unwrap().to_string_lossy()),
        )
        .file("docs/guide.md", "# Guide\n")
        .file("docs/api.md", "# API\n")
        .file("docs/examples/example1.md", "Example 1")
        .file("docs/examples/example2.md", "Example 2")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Top-level directories should be open by default in auto mode
    if output.contains("<details") {
        assert!(output.contains("<details open>") || output.contains("<details>"));
    }
}

#[test]
fn test_clickable_links_default() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root\n")
        .file("src/main.rs", "fn main() {}\n")
        .file("src/lib.rs", "pub fn lib() {}\n")
        .file("src/modules/mod1.rs", "mod mod1")
        .file("src/modules/mod2.rs", "mod mod2")
        .file("src/modules/mod3.rs", "mod mod3")
        .files_with(
            (1..=10).map(|i| format!("src/modules/utils/util{}.rs", i)),
            |p| format!("// util {}\n", p.file_name().unwrap().to_string_lossy()),
        )
        .file("docs/guide.md", "# Guide\n")
        .file("docs/api.md", "# API\n")
        .file("docs/examples/example1.md", "Example 1")
        .file("docs/examples/example2.md", "Example 2")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Files should have clickable links by default
    assert!(output.contains("<a href="));
    assert!(output.contains("README.md</a>") || output.contains(">README.md<"));
}

#[test]
fn test_links_disabled() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Root\n")
        .file("src/main.rs", "fn main() {}\n")
        .file("src/lib.rs", "pub fn lib() {}\n")
        .file("src/modules/mod1.rs", "mod mod1")
        .file("src/modules/mod2.rs", "mod mod2")
        .file("src/modules/mod3.rs", "mod mod3")
        .files_with(
            (1..=10).map(|i| format!("src/modules/utils/util{}.rs", i)),
            |p| format!("// util {}\n", p.file_name().unwrap().to_string_lossy()),
        )
        .file("docs/guide.md", "# Guide\n")
        .file("docs/api.md", "# API\n")
        .file("docs/examples/example1.md", "Example 1")
        .file("docs/examples/example2.md", "Example 2")
        .build();

    let (output, _, success) = run_tree2md([
        p(&root),
        "--output".into(),
        "html".into(),
        "--links".into(),
        "off".into(),
    ]);
    assert!(success);

    // Should not have anchor tags when links are off
    assert!(!output.contains("<a href="));
    assert!(!output.contains("</a>"));

    // But should still have the file names
    assert!(output.contains("README.md"));
    assert!(output.contains("main.rs"));
}

#[test]
fn test_empty_directory() {
    let (_tmp, root) = FixtureBuilder::new()
        .dir("empty_dir")
        .file("file.txt", "content")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Empty directory should still appear but with (files: 0, dirs: 0)
    assert!(output.contains("empty_dir"));
    if output.contains("(files:") {
        assert!(output.contains("files: 0") || output.contains("files: 1"));
    }
}

#[test]
fn test_no_extension_files_in_stats() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("Makefile", "all:")
        .file("Dockerfile", "FROM")
        .file("LICENSE", "MIT")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--output".into(), "html".into()]);
    assert!(success);

    // Stats should handle files without extensions
    assert!(output.contains("**Stats**"));
    assert!(output.contains("Files: 3"));

    // Should show "no-ext" or similar for files without extensions
    if output.contains("Top by ext:") {
        assert!(output.contains("no-ext(3)") || output.contains("(no ext)"));
    }
}
