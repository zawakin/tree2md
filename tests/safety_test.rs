mod fixtures;

use fixtures::{p, run_tree2md, FixtureBuilder};

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

    // Safe mode should be ON by default
    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Normal files should be included
    assert!(output.contains("README.md"));
    assert!(output.contains("main.rs"));

    // Sensitive files should be excluded by default
    assert!(
        !output.contains(".env"),
        ".env should be excluded in safe mode"
    );
    assert!(
        !output.contains(".env.local"),
        ".env.local should be excluded"
    );
    assert!(
        !output.contains(".ssh"),
        ".ssh directory should be excluded"
    );
    assert!(!output.contains("id_rsa"), "SSH keys should be excluded");
    assert!(
        !output.contains("server.pem"),
        "PEM files should be excluded"
    );
    assert!(
        !output.contains("private.key"),
        "Private keys should be excluded"
    );

    // Heavy directories should be excluded
    assert!(
        !output.contains("target/"),
        "target directory should be excluded"
    );
    assert!(
        !output.contains("node_modules/"),
        "node_modules should be excluded"
    );

    // OS files should be excluded
    assert!(
        !output.contains(".DS_Store"),
        ".DS_Store should be excluded"
    );
    assert!(
        !output.contains("Thumbs.db"),
        "Thumbs.db should be excluded"
    );
}

#[test]
fn test_unsafe_mode() {
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

    // With --unsafe flag, sensitive files should be included
    let (output, _, success) = run_tree2md([p(&root), "--unsafe".into()]);
    assert!(success);

    // Normal files should still be included
    assert!(output.contains("README.md"));
    assert!(output.contains("main.rs"));

    // Sensitive files should now be included
    assert!(
        output.contains(".env"),
        ".env should be included in unsafe mode"
    );
    assert!(
        output.contains(".env.local"),
        ".env.local should be included"
    );
    assert!(output.contains(".ssh"), ".ssh directory should be included");
    assert!(output.contains("id_rsa"), "SSH keys should be included");
    assert!(
        output.contains("server.pem"),
        "PEM files should be included"
    );
    assert!(
        output.contains("private.key"),
        "Private keys should be included"
    );

    // Heavy directories should be included
    assert!(
        output.contains("target"),
        "target directory should be included"
    );
    assert!(
        output.contains("node_modules"),
        "node_modules should be included"
    );

    // OS files should be included
    assert!(output.contains(".DS_Store"), ".DS_Store should be included");
    assert!(output.contains("Thumbs.db"), "Thumbs.db should be included");
}

#[test]
fn test_include_pattern_overrides_safe() {
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
        .build();

    // Using -I to explicitly include .env should override safe mode
    let (output, _, success) = run_tree2md([p(&root), "-I".into(), ".env".into()]);
    assert!(success);

    // .env should be included because of explicit include
    assert!(
        output.contains(".env"),
        ".env should be included with -I .env"
    );

    // But .env.local should still be excluded (not explicitly included)
    assert!(
        !output.contains(".env.local"),
        ".env.local should still be excluded"
    );

    // Other sensitive files should still be excluded
    assert!(
        !output.contains("id_rsa"),
        "SSH keys should still be excluded"
    );
}

#[test]
fn test_include_ssh_directory() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Project\n")
        .file("main.rs", "fn main() {}\n")
        .file(".env", "API_KEY=secret123\n")
        .dir(".ssh")
        .file(".ssh/id_rsa", "-----BEGIN RSA PRIVATE KEY-----\n")
        .file(".ssh/id_rsa.pub", "ssh-rsa AAAAB3...\n")
        .file(".ssh/config", "Host *\n")
        .build();

    // Explicitly include .ssh directory
    let (output, _, success) = run_tree2md([p(&root), "-I".into(), ".ssh/**".into()]);
    assert!(success);

    // .ssh directory and its contents should be included
    assert!(output.contains(".ssh"), ".ssh directory should be included");
    assert!(output.contains("id_rsa"), "id_rsa should be included");
    assert!(
        output.contains("id_rsa.pub"),
        "id_rsa.pub should be included"
    );
    assert!(output.contains("config"), "SSH config should be included");

    // Other sensitive files should still be excluded
    assert!(!output.contains(".env"), ".env should still be excluded");
}

#[test]
fn test_safe_mode_with_gitignore() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("main.rs", "fn main() {}")
        .file(".env", "SECRET=123")
        .file("temp.log", "log data")
        .file(".gitignore", "*.log\n")
        .build();

    // Run with default safe mode
    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Normal files included
    assert!(output.contains("main.rs"));

    // .env excluded by safe mode (higher precedence than gitignore)
    assert!(!output.contains(".env"));

    // .log excluded by gitignore
    assert!(!output.contains("temp.log"));

    // .gitignore itself should be visible
    assert!(output.contains(".gitignore"));
}

#[test]
fn test_exclude_pattern_with_safe_mode() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Project\n")
        .file("main.rs", "fn main() {}\n")
        .file(".env", "API_KEY=secret123\n")
        .file(".env.local", "LOCAL_KEY=local456\n")
        .dir(".ssh")
        .file(".ssh/id_rsa", "-----BEGIN RSA PRIVATE KEY-----\n")
        .file(".ssh/id_rsa.pub", "ssh-rsa AAAAB3...\n")
        .file(".ssh/config", "Host *\n")
        .build();

    // Use -X to exclude additional patterns on top of safe mode
    let (output, _, success) = run_tree2md([p(&root), "-X".into(), "*.md".into()]);
    assert!(success);

    // .md files should be excluded by -X
    assert!(
        !output.contains("README.md"),
        "README.md should be excluded by -X *.md"
    );

    // Rust files should still be included
    assert!(output.contains("main.rs"));

    // Safe mode exclusions should still apply
    assert!(!output.contains(".env"));
    assert!(!output.contains(".ssh"));
}

#[test]
fn test_explicit_safe_flag() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("README.md", "# Project\n")
        .file("main.rs", "fn main() {}\n")
        .file(".env", "API_KEY=secret123\n")
        .file(".env.local", "LOCAL_KEY=local456\n")
        .dir(".ssh")
        .file(".ssh/id_rsa", "-----BEGIN RSA PRIVATE KEY-----\n")
        .file(".ssh/id_rsa.pub", "ssh-rsa AAAAB3...\n")
        .file(".ssh/config", "Host *\n")
        .build();

    // --safe flag should work (even though it's default)
    let (output, _, success) = run_tree2md([p(&root), "--safe".into()]);
    assert!(success);

    // Should behave same as default
    assert!(output.contains("README.md"));
    assert!(output.contains("main.rs"));
    assert!(!output.contains(".env"));
    assert!(!output.contains(".ssh"));
}

#[test]
fn test_cache_directories_excluded() {
    let (_tmp, root) = FixtureBuilder::new()
        .file(".cache/data.cache", "cached")
        .file("__pycache__/module.pyc", "bytecode")
        .file("main.py", "print('hello')")
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // Normal file included
    assert!(output.contains("main.py"));

    // Cache directories excluded in safe mode
    assert!(!output.contains(".cache"));
    assert!(!output.contains("__pycache__"));
    assert!(!output.contains(".pyc"));
}

#[test]
fn test_package_lock_files_included() {
    let (_tmp, root) = FixtureBuilder::new()
        .file("package-lock.json", "{}")
        .file("Cargo.lock", "[[package]]")
        .file("yarn.lock", "# yarn")
        .build();

    let (output, _, success) = run_tree2md([p(&root)]);
    assert!(success);

    // These should be included as they're not sensitive
    assert!(output.contains("package-lock.json"));
    assert!(output.contains("Cargo.lock"));
    assert!(output.contains("yarn.lock"));
}
