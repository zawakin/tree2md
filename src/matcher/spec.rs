use crate::cli::Args;

/// Declarative specification of file matching rules
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MatchSpec {
    /// File extensions to include (e.g., [".rs", ".go"])
    pub include_ext: Vec<String>,

    /// Glob patterns to include (e.g., ["**/*.rs", "src/*.go"])
    pub include_glob: Vec<String>,

    /// Glob patterns to exclude (e.g., ["**/target/**", "*.min.js"])
    pub exclude_glob: Vec<String>,

    /// Whether to respect gitignore files
    pub respect_gitignore: bool,

    /// Whether pattern matching is case sensitive
    pub case_sensitive: bool,

    /// Keep directories until pruned (usually true to allow tree building)
    pub keep_dirs_until_pruned: bool,
}

impl Default for MatchSpec {
    fn default() -> Self {
        Self {
            include_ext: Vec::new(),
            include_glob: Vec::new(),
            exclude_glob: Vec::new(),
            respect_gitignore: false,
            case_sensitive: true,
            keep_dirs_until_pruned: true,
        }
    }
}

impl MatchSpec {
    /// Create a new MatchSpec with default values
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Normalize a glob pattern to be recursive if it doesn't contain path separators
    /// For example: "*.rs" becomes "**/*.rs" to match files at any depth
    fn normalize_pattern(pattern: &str) -> String {
        if !pattern.contains('/') {
            format!("**/{}", pattern)
        } else {
            pattern.to_string()
        }
    }

    /// Create a MatchSpec from CLI arguments
    pub fn from_args(args: &Args) -> Self {
        let include_ext = if let Some(ref ext_str) = args.include_ext {
            ext_str
                .split(',')
                .map(|s| {
                    let trimmed = s.trim();
                    if trimmed.starts_with('.') {
                        trimmed.to_string()
                    } else {
                        format!(".{}", trimmed)
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        // Normalize patterns to be recursive by default
        let include_glob = args
            .find_patterns
            .iter()
            .map(|p| Self::normalize_pattern(p))
            .collect();

        Self {
            include_ext,
            include_glob,
            exclude_glob: Vec::new(), // Could be extended with --exclude flag
            respect_gitignore: !args.no_gitignore,
            case_sensitive: true, // Could be extended with --ignore-case flag
            keep_dirs_until_pruned: true,
        }
    }

    /// Check if any inclusion rules are specified
    pub fn has_includes(&self) -> bool {
        !self.include_ext.is_empty() || !self.include_glob.is_empty()
    }

    /// Check if any exclusion rules are specified
    #[allow(dead_code)]
    pub fn has_excludes(&self) -> bool {
        !self.exclude_glob.is_empty() || self.respect_gitignore
    }

    /// Builder methods for fluent API
    #[allow(dead_code)]
    pub fn with_include_ext(mut self, extensions: Vec<String>) -> Self {
        self.include_ext = extensions;
        self
    }

    #[allow(dead_code)]
    pub fn with_include_glob(mut self, patterns: Vec<String>) -> Self {
        // Normalize patterns to be recursive by default
        self.include_glob = patterns
            .iter()
            .map(|p| Self::normalize_pattern(p))
            .collect();
        self
    }

    #[allow(dead_code)]
    pub fn with_exclude_glob(mut self, patterns: Vec<String>) -> Self {
        // Normalize patterns for exclude as well
        self.exclude_glob = patterns
            .iter()
            .map(|p| Self::normalize_pattern(p))
            .collect();
        self
    }

    #[allow(dead_code)]
    pub fn with_gitignore(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }

    #[allow(dead_code)]
    pub fn with_case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_spec() {
        let spec = MatchSpec::default();
        assert!(spec.include_ext.is_empty());
        assert!(spec.include_glob.is_empty());
        assert!(spec.exclude_glob.is_empty());
        assert!(!spec.respect_gitignore);
        assert!(spec.case_sensitive);
        assert!(spec.keep_dirs_until_pruned);
    }

    #[test]
    fn test_has_includes() {
        let spec = MatchSpec::new();
        assert!(!spec.has_includes());

        let spec = spec.with_include_ext(vec![".rs".to_string()]);
        assert!(spec.has_includes());

        let spec = MatchSpec::new().with_include_glob(vec!["*.rs".to_string()]);
        assert!(spec.has_includes());
    }

    #[test]
    fn test_builder_methods() {
        let spec = MatchSpec::new()
            .with_include_ext(vec![".rs".to_string(), ".go".to_string()])
            .with_include_glob(vec!["src/**/*.rs".to_string()])
            .with_exclude_glob(vec!["target/**".to_string()])
            .with_gitignore(true)
            .with_case_sensitive(false);

        assert_eq!(spec.include_ext.len(), 2);
        assert_eq!(spec.include_glob.len(), 1);
        assert_eq!(spec.exclude_glob.len(), 1);
        assert!(spec.respect_gitignore);
        assert!(!spec.case_sensitive);
    }

    #[test]
    fn test_pattern_normalization() {
        // Test that simple patterns are normalized to be recursive
        let spec = MatchSpec::new().with_include_glob(vec![
            "*.rs".to_string(),     // Should be normalized to **/*.rs
            "foo.txt".to_string(),  // Should be normalized to **/foo.txt
            "src/*.go".to_string(), // Should NOT be normalized (has /)
            "**/*.md".to_string(),  // Already recursive, no change
        ]);

        assert_eq!(spec.include_glob.len(), 4);
        assert_eq!(spec.include_glob[0], "**/*.rs");
        assert_eq!(spec.include_glob[1], "**/foo.txt");
        assert_eq!(spec.include_glob[2], "src/*.go"); // Not normalized
        assert_eq!(spec.include_glob[3], "**/*.md");

        // Test exclude patterns are also normalized
        let spec = MatchSpec::new().with_exclude_glob(vec![
            "*.tmp".to_string(),  // Should be normalized
            "build/".to_string(), // Should NOT be normalized (has /)
        ]);

        assert_eq!(spec.exclude_glob[0], "**/*.tmp");
        assert_eq!(spec.exclude_glob[1], "build/");
    }
}
