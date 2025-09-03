use super::{MatchSpec, RelPath};
use crate::safety::SafetyPreset;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::collections::HashSet;
use std::io;
use std::path::Path;

/// Selection decision for a path
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    /// Include this path in the output
    Include,
    /// Exclude this path from the output
    Exclude,
    /// Prune this directory (don't descend into it)
    PruneDir,
}

/// Compiled matcher engine that evaluates paths against rules
pub struct MatcherEngine {
    /// Compiled extension set for fast lookups
    include_ext_set: HashSet<String>,

    /// Original include glob patterns (for directory checking)
    include_glob: Vec<String>,

    /// Compiled include glob patterns
    include_globset: Option<GlobSet>,

    /// Compiled exclude glob patterns
    exclude_globset: Option<GlobSet>,

    /// Gitignore rules if enabled
    gitignore: Option<Gitignore>,

    /// Safety preset for excluding sensitive files
    safety_preset: Option<SafetyPreset>,

    /// Whether we have any include rules
    has_includes: bool,

    /// Whether matching is case sensitive
    case_sensitive: bool,
}

impl MatcherEngine {
    /// Compile a MatchSpec into an optimized MatcherEngine
    pub fn compile(spec: &MatchSpec, root: &Path) -> io::Result<Self> {
        // Build extension set
        let include_ext_set: HashSet<String> = spec
            .include_ext
            .iter()
            .map(|ext| {
                if spec.case_sensitive {
                    ext.clone()
                } else {
                    ext.to_lowercase()
                }
            })
            .collect();

        // Build include globset
        let include_globset = if !spec.include_glob.is_empty() {
            let mut builder = GlobSetBuilder::new();
            for pattern in &spec.include_glob {
                let glob = Glob::new(pattern).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Invalid include glob pattern '{}': {}", pattern, e),
                    )
                })?;
                builder.add(glob);
            }
            Some(builder.build().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Failed to build include globset: {}", e),
                )
            })?)
        } else {
            None
        };

        // Build exclude globset
        let exclude_globset = if !spec.exclude_glob.is_empty() {
            let mut builder = GlobSetBuilder::new();
            for pattern in &spec.exclude_glob {
                let glob = Glob::new(pattern).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Invalid exclude glob pattern '{}': {}", pattern, e),
                    )
                })?;
                builder.add(glob);
            }
            Some(builder.build().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Failed to build exclude globset: {}", e),
                )
            })?)
        } else {
            None
        };

        // Build gitignore if needed
        let gitignore = if spec.respect_gitignore {
            let mut builder = GitignoreBuilder::new(root);

            // Add .gitignore from the root and parent directories
            let mut current = root;
            loop {
                let gitignore_path = current.join(".gitignore");
                if gitignore_path.exists() {
                    builder.add(gitignore_path);
                }

                if let Some(parent) = current.parent() {
                    current = parent;
                } else {
                    break;
                }
            }

            // Also check for global gitignore
            if let Some(home) = dirs::home_dir() {
                let global_gitignore = home.join(".gitignore");
                if global_gitignore.exists() {
                    builder.add(global_gitignore);
                }
            }

            Some(builder.build().map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Failed to build gitignore: {}", e),
                )
            })?)
        } else {
            None
        };

        // Create safety preset if enabled
        let safety_preset = if spec.use_safety_preset {
            Some(SafetyPreset::new())
        } else {
            None
        };

        Ok(Self {
            include_ext_set,
            include_glob: spec.include_glob.clone(),
            include_globset,
            exclude_globset,
            gitignore,
            safety_preset,
            has_includes: spec.has_includes(),
            case_sensitive: spec.case_sensitive,
        })
    }

    /// Select whether to include, exclude, or prune a file
    pub fn select_file(&self, rel_path: &RelPath) -> Selection {
        let path_str = rel_path.as_match_str();

        // Precedence: include > exclude > gitignore > safe

        // Priority 1: Include patterns (highest priority)
        if self.matches_include_rules(&path_str, rel_path) {
            return Selection::Include;
        }

        // Priority 2: Exclude patterns
        if self.matches_exclude_rules(&path_str, rel_path) {
            return Selection::Exclude;
        }

        // Priority 3: Gitignore rules
        if let Some(ref gitignore) = self.gitignore {
            let path_buf = rel_path.to_path_buf();
            if gitignore.matched(&path_buf, false).is_ignore() {
                return Selection::Exclude;
            }
        }

        // Priority 4: Safety preset (lowest priority)
        if let Some(ref safety) = self.safety_preset {
            if safety.matches(path_str.as_ref()) {
                return Selection::Exclude;
            }
        }

        // If we have include patterns but file didn't match, exclude it
        if self.has_includes {
            return Selection::Exclude;
        }

        // Default: include if no rules match
        Selection::Include
    }

    /// Select whether to include, exclude, or prune a directory
    pub fn select_dir(&self, rel_path: &RelPath) -> Selection {
        let path_str = rel_path.as_match_str();

        // Always exclude .git directory
        if path_str == ".git" || path_str.starts_with(".git/") {
            return Selection::PruneDir;
        }

        // Same precedence as files: include > exclude > gitignore > safe

        // Priority 1: Include patterns can override everything
        // Check both if the directory itself matches OR if it may contain matching files
        if self.matches_include_rules(&path_str, rel_path)
            || self.dir_may_contain_includes(&path_str)
        {
            return Selection::Include;
        }

        // Priority 2: Exclude patterns
        if let Some(ref exclude_globset) = self.exclude_globset {
            // For directory matching, try both with and without trailing slash
            if exclude_globset.is_match(path_str.as_ref())
                || exclude_globset.is_match(format!("{}/", path_str))
            {
                return Selection::PruneDir;
            }
        }

        // Priority 3: Gitignore for directories
        if let Some(ref gitignore) = self.gitignore {
            let path_buf = rel_path.to_path_buf();
            if gitignore.matched(&path_buf, true).is_ignore() {
                return Selection::PruneDir;
            }
        }

        // Priority 4: Safety preset for directories
        if let Some(ref safety) = self.safety_preset {
            if safety.matches(path_str.as_ref()) || safety.matches(&format!("{}/", path_str)) {
                return Selection::PruneDir;
            }
        }

        // Don't prune directories by default - we need to check their contents
        Selection::Include
    }

    /// Check if a path matches any include rules
    fn matches_include_rules(&self, path_str: &str, rel_path: &RelPath) -> bool {
        // Check extension matching
        if !self.include_ext_set.is_empty() {
            let path_buf = rel_path.to_path_buf();
            if let Some(ext) = path_buf.extension() {
                let ext_str = format!(".{}", ext.to_string_lossy());
                let ext_to_check = if self.case_sensitive {
                    ext_str
                } else {
                    ext_str.to_lowercase()
                };

                if self.include_ext_set.contains(&ext_to_check) {
                    return true;
                }
            }
        }

        // Check glob patterns
        if let Some(ref include_globset) = self.include_globset {
            if include_globset.is_match(path_str) {
                return true;
            }
        }

        false
    }

    /// Check if a directory potentially contains files that match include rules
    fn dir_may_contain_includes(&self, path_str: &str) -> bool {
        if let Some(ref include_globset) = self.include_globset {
            // Check if any include pattern would match files inside this directory
            // For example, if include pattern is ".ssh/**", it matches files in .ssh/
            let with_wildcard = format!("{}/**", path_str);

            // Check if the pattern matches the directory itself or content within it
            for pattern in &self.include_glob {
                // If pattern is exactly "dir/**" and we are "dir", we match
                if pattern == &with_wildcard || pattern == &format!("**/{}", with_wildcard) {
                    return true;
                }
                // Also check with globset if any file inside would match
                // Test with a hypothetical file inside the directory
                let test_file = format!("{}/test", path_str);
                if include_globset.is_match(&test_file) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a path matches any exclude rules
    fn matches_exclude_rules(&self, path_str: &str, rel_path: &RelPath) -> bool {
        // Check gitignore
        if let Some(ref gitignore) = self.gitignore {
            let path_buf = rel_path.to_path_buf();
            if gitignore.matched(&path_buf, false).is_ignore() {
                return true;
            }
        }

        // Check exclude globs
        if let Some(ref exclude_globset) = self.exclude_globset {
            if exclude_globset.is_match(path_str) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_include_extensions() {
        let spec = MatchSpec::new().with_include_ext(vec![".rs".to_string(), ".go".to_string()]);

        let temp_dir = TempDir::new().unwrap();
        let engine = MatcherEngine::compile(&spec, temp_dir.path()).unwrap();

        let rs_file = RelPath::from_relative("src/main.rs");
        assert_eq!(engine.select_file(&rs_file), Selection::Include);

        let go_file = RelPath::from_relative("pkg/server.go");
        assert_eq!(engine.select_file(&go_file), Selection::Include);

        let txt_file = RelPath::from_relative("readme.txt");
        assert_eq!(engine.select_file(&txt_file), Selection::Exclude);
    }

    #[test]
    fn test_include_globs() {
        let spec =
            MatchSpec::new().with_include_glob(vec!["src/**/*.rs".to_string(), "*.md".to_string()]);

        let temp_dir = TempDir::new().unwrap();
        let engine = MatcherEngine::compile(&spec, temp_dir.path()).unwrap();

        let src_rs = RelPath::from_relative("src/main.rs");
        assert_eq!(engine.select_file(&src_rs), Selection::Include);

        let nested_rs = RelPath::from_relative("src/module/lib.rs");
        assert_eq!(engine.select_file(&nested_rs), Selection::Include);

        let readme = RelPath::from_relative("README.md");
        assert_eq!(engine.select_file(&readme), Selection::Include);

        let other_rs = RelPath::from_relative("test/main.rs");
        assert_eq!(engine.select_file(&other_rs), Selection::Exclude);
    }

    #[test]
    fn test_exclude_globs() {
        let spec = MatchSpec::new()
            .with_exclude_glob(vec!["**/target/**".to_string(), "*.tmp".to_string()]);

        let temp_dir = TempDir::new().unwrap();
        let engine = MatcherEngine::compile(&spec, temp_dir.path()).unwrap();

        let normal_file = RelPath::from_relative("src/main.rs");
        assert_eq!(engine.select_file(&normal_file), Selection::Include);

        let target_file = RelPath::from_relative("target/debug/app");
        assert_eq!(engine.select_file(&target_file), Selection::Exclude);

        let tmp_file = RelPath::from_relative("cache.tmp");
        assert_eq!(engine.select_file(&tmp_file), Selection::Exclude);

        // Test directory pruning
        let target_dir = RelPath::from_relative("target");
        assert_eq!(engine.select_dir(&target_dir), Selection::PruneDir);
    }

    #[test]
    fn test_precedence() {
        // Include rules take precedence over exclude rules
        let spec = MatchSpec::new()
            .with_include_glob(vec!["**/*.rs".to_string()])
            .with_exclude_glob(vec!["**/test/**".to_string()]);

        let temp_dir = TempDir::new().unwrap();
        let engine = MatcherEngine::compile(&spec, temp_dir.path()).unwrap();

        let src_rs = RelPath::from_relative("src/main.rs");
        assert_eq!(engine.select_file(&src_rs), Selection::Include);

        // Even though test_main.rs is in the exclude pattern **/test/**,
        // it matches the include pattern **/*.rs, and include takes precedence
        let test_rs = RelPath::from_relative("test/test_main.rs");
        assert_eq!(engine.select_file(&test_rs), Selection::Include);
    }

    #[test]
    fn test_hidden_files() {
        // Hidden files are now handled by WalkBuilder, not MatcherEngine
        let spec = MatchSpec::new();
        let temp_dir = TempDir::new().unwrap();
        let engine = MatcherEngine::compile(&spec, temp_dir.path()).unwrap();

        // Hidden filtering is done by WalkBuilder, so MatcherEngine always includes
        let hidden_file = RelPath::from_relative(".gitignore");
        assert_eq!(engine.select_file(&hidden_file), Selection::Include);

        // .git directory is always excluded
        let git_dir = RelPath::from_relative(".git");
        assert_eq!(engine.select_dir(&git_dir), Selection::PruneDir);

        // Other hidden directories are included (filtered by WalkBuilder)
        let other_hidden_dir = RelPath::from_relative(".config");
        assert_eq!(engine.select_dir(&other_hidden_dir), Selection::Include);
    }
}
