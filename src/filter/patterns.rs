use glob::{MatchOptions, Pattern};
use std::io;

/// Normalize a glob pattern to be recursive if it doesn't contain path separators
/// For example: "*.rs" becomes "**/*.rs" to match files at any depth
fn normalize_pattern(pattern: &str) -> String {
    if !pattern.contains('/') {
        format!("**/{}", pattern)
    } else {
        pattern.to_string()
    }
}

/// Compile glob patterns from strings
pub fn compile_patterns(pattern_strings: &[String]) -> io::Result<Vec<Pattern>> {
    let mut patterns = Vec::new();
    for pattern_str in pattern_strings {
        // Normalize patterns to be recursive by default
        let normalized = normalize_pattern(pattern_str);
        match Pattern::new(&normalized) {
            Ok(pattern) => patterns.push(pattern),
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid glob pattern '{}': {}", normalized, e),
                ));
            }
        }
    }
    Ok(patterns)
}

/// Pure function to check if a relative path matches any of the patterns
///
/// # Arguments
/// * `relative_path` - The path relative to the search root (e.g., "src/main.rs", "lib.rs")
/// * `patterns` - The compiled glob patterns
///
/// # Returns
/// True if the path matches any pattern
pub fn path_matches_any_pattern(relative_path: &str, patterns: &[Pattern]) -> bool {
    // Configure matching options
    // require_literal_separator = true means '*' won't match '/'
    // This makes '*.rs' only match files in the current directory
    let options = MatchOptions {
        case_sensitive: true,
        require_literal_separator: true,
        require_literal_leading_dot: false,
    };

    for pattern in patterns {
        if pattern.matches_with(relative_path, options) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_patterns() {
        let patterns = compile_patterns(&["*.rs".to_string(), "src/**/*.go".to_string()]).unwrap();
        assert_eq!(patterns.len(), 2);
    }

    #[test]
    fn test_invalid_pattern() {
        let result = compile_patterns(&["[".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_star_pattern() {
        // Test that '*.rs' now matches files recursively (after normalization to '**/*.rs')
        let patterns = compile_patterns(&["*.rs".to_string()]).unwrap();

        // Should match files in root
        assert!(
            path_matches_any_pattern("main.rs", &patterns),
            "Should match main.rs"
        );
        assert!(
            path_matches_any_pattern("lib.rs", &patterns),
            "Should match lib.rs"
        );
        assert!(
            path_matches_any_pattern("test.rs", &patterns),
            "Should match test.rs"
        );

        // Should NOW match files in subdirectories (recursive behavior)
        assert!(
            path_matches_any_pattern("src/main.rs", &patterns),
            "Should match src/main.rs (recursive)"
        );
        assert!(
            path_matches_any_pattern("module/lib.rs", &patterns),
            "Should match module/lib.rs (recursive)"
        );
        assert!(
            path_matches_any_pattern("a/b/c/test.rs", &patterns),
            "Should match deeply nested (recursive)"
        );

        // Should NOT match non-.rs files
        assert!(
            !path_matches_any_pattern("main.txt", &patterns),
            "Should NOT match main.txt"
        );
        assert!(
            !path_matches_any_pattern("lib", &patterns),
            "Should NOT match lib without extension"
        );
    }

    #[test]
    fn test_double_star_pattern() {
        // Test that '**/*.rs' matches files in any directory
        let patterns = compile_patterns(&["**/*.rs".to_string()]).unwrap();

        // Should match files at any level
        assert!(
            path_matches_any_pattern("main.rs", &patterns),
            "Should match main.rs"
        );
        assert!(
            path_matches_any_pattern("src/main.rs", &patterns),
            "Should match src/main.rs"
        );
        assert!(
            path_matches_any_pattern("a/b/c/test.rs", &patterns),
            "Should match deeply nested"
        );

        // Should NOT match non-.rs files
        assert!(
            !path_matches_any_pattern("main.txt", &patterns),
            "Should NOT match main.txt"
        );
        assert!(
            !path_matches_any_pattern("src/lib.go", &patterns),
            "Should NOT match lib.go"
        );
    }

    #[test]
    fn test_directory_specific_pattern() {
        // Test patterns like 'src/*.rs' - these should NOT be normalized
        let patterns = compile_patterns(&["src/*.rs".to_string()]).unwrap();

        // Should match files in src/ directory only
        assert!(
            path_matches_any_pattern("src/main.rs", &patterns),
            "Should match src/main.rs"
        );
        assert!(
            path_matches_any_pattern("src/lib.rs", &patterns),
            "Should match src/lib.rs"
        );

        // Should NOT match files in root
        assert!(
            !path_matches_any_pattern("main.rs", &patterns),
            "Should NOT match root main.rs"
        );
        assert!(
            !path_matches_any_pattern("lib.rs", &patterns),
            "Should NOT match root lib.rs"
        );

        // Should NOT match files in nested directories (pattern has '/' so not normalized)
        assert!(
            !path_matches_any_pattern("src/module/test.rs", &patterns),
            "Should NOT match src/module/test.rs (not recursive)"
        );
        assert!(
            !path_matches_any_pattern("other/main.rs", &patterns),
            "Should NOT match other/main.rs"
        );
    }

    #[test]
    fn test_multiple_patterns() {
        // Test multiple patterns together
        let patterns = compile_patterns(&[
            "*.md".to_string(),        // Will be normalized to **/*.md
            "src/*.rs".to_string(),     // Will NOT be normalized (contains /)
            "test/**/*.txt".to_string(), // Already recursive
        ])
        .unwrap();

        // First pattern: *.md now matches recursively
        assert!(
            path_matches_any_pattern("README.md", &patterns),
            "Should match README.md"
        );
        assert!(
            path_matches_any_pattern("docs/README.md", &patterns),
            "Should NOW match docs/README.md (recursive)"
        );

        // Second pattern: src/*.rs (not normalized since it has /)
        assert!(
            path_matches_any_pattern("src/main.rs", &patterns),
            "Should match src/main.rs"
        );
        assert!(
            !path_matches_any_pattern("src/module/lib.rs", &patterns),
            "Should NOT match src/module/lib.rs (not recursive)"
        );

        // Third pattern: test/**/*.txt (already recursive)
        assert!(
            path_matches_any_pattern("test/data.txt", &patterns),
            "Should match test/data.txt"
        );
        assert!(
            path_matches_any_pattern("test/fixtures/sample.txt", &patterns),
            "Should match test/fixtures/sample.txt"
        );
        assert!(
            !path_matches_any_pattern("data.txt", &patterns),
            "Should NOT match root data.txt (pattern requires test/ prefix)"
        );
    }

    #[test]
    fn test_pattern_normalization() {
        // Test that patterns without '/' are normalized to be recursive
        
        // Simple wildcard patterns should become recursive
        let patterns = compile_patterns(&["*.rs".to_string()]).unwrap();
        assert!(
            path_matches_any_pattern("main.rs", &patterns),
            "*.rs should match root files"
        );
        assert!(
            path_matches_any_pattern("src/lib.rs", &patterns),
            "*.rs should match nested files after normalization"
        );
        assert!(
            path_matches_any_pattern("deep/nested/file.rs", &patterns),
            "*.rs should match deeply nested files"
        );
        
        // Literal filename patterns should also become recursive
        let patterns = compile_patterns(&["foo.rs".to_string()]).unwrap();
        assert!(
            path_matches_any_pattern("foo.rs", &patterns),
            "foo.rs should match in root"
        );
        assert!(
            path_matches_any_pattern("src/foo.rs", &patterns),
            "foo.rs should match in subdirectory"
        );
        assert!(
            !path_matches_any_pattern("bar.rs", &patterns),
            "foo.rs should not match bar.rs"
        );
        
        // Patterns with '/' should NOT be normalized
        let patterns = compile_patterns(&["src/*.rs".to_string()]).unwrap();
        assert!(
            path_matches_any_pattern("src/main.rs", &patterns),
            "src/*.rs should match files in src/"
        );
        assert!(
            !path_matches_any_pattern("src/nested/main.rs", &patterns),
            "src/*.rs should NOT match nested files (not normalized)"
        );
        assert!(
            !path_matches_any_pattern("main.rs", &patterns),
            "src/*.rs should NOT match root files"
        );
    }

    #[test]
    fn test_edge_cases() {
        // Test edge cases and special characters
        let patterns = compile_patterns(&["*.rs".to_string()]).unwrap();

        // Empty path
        assert!(
            !path_matches_any_pattern("", &patterns),
            "Should NOT match empty path"
        );

        // Just extension - actually * matches zero or more chars, so .rs should match
        assert!(
            path_matches_any_pattern(".rs", &patterns),
            "Should match .rs (hidden file with rs extension)"
        );

        // Hidden files
        assert!(
            path_matches_any_pattern(".hidden.rs", &patterns),
            "Should match .hidden.rs"
        );

        // Paths with spaces (if supported)
        assert!(
            path_matches_any_pattern("my file.rs", &patterns),
            "Should match 'my file.rs'"
        );
        assert!(
            path_matches_any_pattern("dir/my file.rs", &patterns),
            "Should NOW match 'dir/my file.rs' (recursive)"
        );
    }
}
