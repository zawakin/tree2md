mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

#[test]
fn test_pipe_tree_structure() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("src/main.rs", "fn main() {}")
        .file("src/lib.rs", "pub fn lib() {}")
        .file("Cargo.toml", "[package]\nname = \"test\"")
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Should start with "."
    assert!(output.starts_with('.'), "Should start with root marker '.'");

    // Should contain tree characters
    assert!(
        output.contains("├── ") || output.contains("└── "),
        "Should contain tree branch characters"
    );

    // Should contain filenames
    assert!(output.contains("main.rs"));
    assert!(output.contains("lib.rs"));
    assert!(output.contains("Cargo.toml"));

    // Directories should end with /
    assert!(output.contains("src/"));
}

#[test]
fn test_pipe_tree_nesting() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("src/models/user.rs", "struct User {}")
        .file("src/main.rs", "fn main() {}")
        .file("README.md", "# Test")
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Should have continuation characters for nested dirs
    assert!(
        output.contains("│   "),
        "Should have vertical continuation for nested items"
    );
}

#[test]
fn test_pipe_loc_display() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {\n    println!(\"Hello\");\n}\n")
        .file("lib.rs", "pub fn lib() {}\n")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--loc".into(), "fast".into()]);
    assert!(success);

    // Should show line counts in pipe mode
    assert!(
        output.contains("lines)"),
        "Should show line count annotations"
    );
    assert!(
        output.contains("(3 lines)") || output.contains("(1 lines)"),
        "Should show specific line counts"
    );
}

#[test]
fn test_pipe_loc_off() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "--loc".into(), "off".into()]);
    assert!(success);

    // Should not show line counts when LOC is off
    assert!(
        !output.contains("lines)"),
        "Should not show line counts when off"
    );
}

#[test]
fn test_pipe_contents_flag() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("hello.rs", "fn hello() {\n    println!(\"Hello!\");\n}\n")
        .file("README.md", "# Test Project\n")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "-c".into()]);
    assert!(success);

    // Should contain tree structure
    assert!(output.contains("hello.rs"));
    assert!(output.contains("README.md"));

    // Should contain file contents in code fences
    assert!(output.contains("```rust"), "Should have rust code fence");
    assert!(
        output.contains("fn hello()"),
        "Should contain Rust file content"
    );
    assert!(
        output.contains("```markdown"),
        "Should have markdown code fence"
    );
    assert!(
        output.contains("# Test Project"),
        "Should contain markdown file content"
    );
}

#[test]
fn test_pipe_contents_binary_skip() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("code.rs", "fn main() {}\n")
        .file("image.png", "fake png binary content")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "-c".into()]);
    assert!(success);

    // Should include code file content
    assert!(output.contains("```rust"));
    assert!(output.contains("fn main()"));

    // Should skip binary file content (no code fence for png)
    // But the filename should still appear in the tree
    assert!(output.contains("image.png"), "Binary file in tree");
    assert!(
        !output.contains("```png"),
        "Should not have code fence for binary"
    );
}

#[test]
fn test_pipe_contents_lang_hint() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.py", "print('hello')\n")
        .file("config.toml", "[section]\nkey = \"value\"\n")
        .file("data.json", "{\"key\": \"value\"}\n")
        .build();

    let (output, _, success) = run_tree2md([p(&root), "-c".into()]);
    assert!(success);

    assert!(output.contains("```python"), "Should detect Python");
    assert!(output.contains("```toml"), "Should detect TOML");
    assert!(output.contains("```json"), "Should detect JSON");
}

#[test]
fn test_pipe_no_emoji() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .dir("src")
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Pipe mode should never show emojis
    assert!(!output.contains("🦀"), "No emojis in pipe mode");
    assert!(!output.contains("📁"), "No dir emoji in pipe mode");
}

#[test]
fn test_pipe_stats_present() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file("lib.rs", "pub fn lib() {}")
        .dir("src")
        .build();

    // Stats should be shown by default (--stats full)
    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    assert!(
        output.contains("Totals") || output.contains("Stats"),
        "Should show stats by default"
    );
}
