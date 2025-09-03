use crate::cli::Args;

/// Declarative specification of file matching rules
#[derive(Debug, Clone)]
pub struct MatchSpec {
    /// File extensions to include (e.g., [".rs", ".go"])
    pub include_ext: Vec<String>,

    /// Glob patterns to include (e.g., ["**/*.rs", "src/*.go"])
    pub include_glob: Vec<String>,

    /// Glob patterns to exclude (e.g., ["**/target/**", "*.min.js"])
    pub exclude_glob: Vec<String>,

    /// Whether to respect gitignore files
    pub respect_gitignore: bool,

    /// Whether to apply safety presets (exclude sensitive files)
    pub use_safety_preset: bool,

    /// Whether pattern matching is case sensitive
    pub case_sensitive: bool,

    /// Keep directories until pruned (usually true to allow tree building)
    pub _keep_dirs_until_pruned: bool,
}

impl Default for MatchSpec {
    fn default() -> Self {
        Self {
            include_ext: Vec::new(),
            include_glob: Vec::new(),
            exclude_glob: Vec::new(),
            respect_gitignore: false,
            use_safety_preset: true, // Default to safe mode ON
            case_sensitive: true,
            _keep_dirs_until_pruned: true,
        }
    }
}

impl MatchSpec {
    /// Create a new MatchSpec with default values
    #[allow(dead_code)] // Used in tests
    pub fn new() -> Self {
        Self::default()
    }

    /// Normalize a glob pattern to be recursive if it doesn't contain path separators
    /// For example: "*.rs" becomes "**/*.rs" to match files at any depth
    /// For directory names like "specs", it becomes "specs/**" to match everything under that directory
    fn normalize_pattern(pattern: &str) -> String {
        if !pattern.contains('/') {
            // Check if this looks like a directory name (no wildcards or extensions)
            if !pattern.contains('*') && !pattern.contains('.') {
                // Likely a directory name, match everything under it
                format!("{}/**", pattern)
            } else {
                // File pattern, make it recursive
                format!("**/{}", pattern)
            }
        } else {
            pattern.to_string()
        }
    }

    /// Create a MatchSpec from CLI arguments
    pub fn from_args(args: &Args, target_path: &std::path::Path) -> Self {
        // No more include_ext in new CLI, use include patterns instead
        let include_ext = Vec::new();

        // Use the new include patterns from -I/--include
        let include_glob = args
            .include
            .iter()
            .map(|p| Self::normalize_pattern(p))
            .collect();

        // Use the new exclude patterns from -X/--exclude
        let exclude_glob = args
            .exclude
            .iter()
            .map(|p| Self::normalize_pattern(p))
            .collect();

        // Handle gitignore based on the new use_gitignore mode
        let respect_gitignore = match args.use_gitignore {
            crate::cli::UseGitignoreMode::Always => true,
            crate::cli::UseGitignoreMode::Never => false,
            crate::cli::UseGitignoreMode::Auto => {
                // In auto mode, respect gitignore if .git exists in the target directory
                target_path.join(".git").exists()
            }
        };

        Self {
            include_ext,
            include_glob,
            exclude_glob,
            respect_gitignore,
            use_safety_preset: args.is_safe_mode(),
            case_sensitive: true, // Could be extended with --ignore-case flag
            _keep_dirs_until_pruned: true,
        }
    }

    /// Check if any inclusion rules are specified
    pub fn has_includes(&self) -> bool {
        !self.include_ext.is_empty() || !self.include_glob.is_empty()
    }

    /// Builder methods for fluent API
    #[allow(dead_code)] // Used in tests
    pub fn with_include_ext(mut self, extensions: Vec<String>) -> Self {
        self.include_ext = extensions;
        self
    }

    #[allow(dead_code)] // Used in tests
    pub fn with_include_glob(mut self, patterns: Vec<String>) -> Self {
        // Normalize patterns to be recursive by default
        self.include_glob = patterns
            .iter()
            .map(|p| Self::normalize_pattern(p))
            .collect();
        self
    }

    #[allow(dead_code)] // Used in tests
    pub fn with_exclude_glob(mut self, patterns: Vec<String>) -> Self {
        // Normalize patterns for exclude as well
        self.exclude_glob = patterns
            .iter()
            .map(|p| Self::normalize_pattern(p))
            .collect();
        self
    }

    #[allow(dead_code)] // Used in tests
    pub fn with_gitignore(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }

    #[allow(dead_code)] // Used in tests
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
        assert!(spec._keep_dirs_until_pruned);
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
