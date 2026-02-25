use super::{MatchSpec, RelPath};
use crate::safety::SafetyPreset;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::collections::HashSet;
use std::io;
use std::path::Path;
use std::path::PathBuf;

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

    /// Gitignore rules: list of (scope_dir relative to root, compiled gitignore).
    /// Each entry applies only to paths under its scope directory.
    /// A scope of "" means root-level (applies to everything).
    gitignore_layers: Vec<(String, Gitignore)>,

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

        // Build gitignore layers if needed.
        // Each .gitignore file becomes a separate layer with its own scope,
        // because the `ignore` crate's Gitignore::matched() does not enforce
        // directory scoping on its own.
        let gitignore_layers = if spec.respect_gitignore {
            let mut layers: Vec<(String, Gitignore)> = Vec::new();

            // Root-level layer: collects patterns from root/.gitignore,
            // parent directories, and global gitignore.
            // These all apply to everything (scope = "").
            let mut root_builder = GitignoreBuilder::new(root);
            let mut has_root_patterns = false;

            // Walk upward from root to collect ancestor .gitignore files
            let mut current = root;
            loop {
                let gitignore_path = current.join(".gitignore");
                if gitignore_path.exists() {
                    root_builder.add(gitignore_path);
                    has_root_patterns = true;
                }
                if let Some(parent) = current.parent() {
                    current = parent;
                } else {
                    break;
                }
            }

            // .git/info/exclude: per-repo exclude patterns (standard git mechanism)
            let git_info_exclude = root.join(".git/info/exclude");
            if git_info_exclude.exists() {
                root_builder.add(git_info_exclude);
                has_root_patterns = true;
            }

            // Global gitignore: ~/.config/git/ignore (Git 2.20+), fallback ~/.gitignore
            if let Some(home) = dirs::home_dir() {
                let xdg_gitignore = home.join(".config/git/ignore");
                let legacy_gitignore = home.join(".gitignore");
                if xdg_gitignore.exists() {
                    root_builder.add(xdg_gitignore);
                    has_root_patterns = true;
                } else if legacy_gitignore.exists() {
                    root_builder.add(legacy_gitignore);
                    has_root_patterns = true;
                }
            }

            if has_root_patterns {
                let gi = root_builder.build().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Failed to build root gitignore: {}", e),
                    )
                })?;
                layers.push((String::new(), gi));
            }

            // Nested layers: each subdirectory .gitignore gets its own Gitignore
            for gitignore_path in Self::collect_nested_gitignores(root) {
                let dir = gitignore_path.parent().unwrap();
                let scope = dir
                    .strip_prefix(root)
                    .unwrap_or(Path::new(""))
                    .to_string_lossy()
                    .replace('\\', "/");

                let mut builder = GitignoreBuilder::new(dir);
                builder.add(&gitignore_path);
                let gi = builder.build().map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Failed to build gitignore for {}: {}", scope, e),
                    )
                })?;
                layers.push((scope, gi));
            }

            layers
        } else {
            Vec::new()
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
            gitignore_layers,
            safety_preset,
            has_includes: spec.has_includes(),
            case_sensitive: spec.case_sensitive,
        })
    }

    /// Select whether to include, exclude, or prune a file
    ///
    /// Priority order:
    /// 1. If has_includes and file doesn't match any include → Exclude
    /// 2. If file matches a path-specific include (e.g., `vendor/**/*.py`) → Include
    ///    (path-specific includes explicitly target files and override exclude)
    /// 3. If file matches exclude → Exclude (narrows generic includes like `**/*.rs`)
    /// 4. If file matched a generic include → Include (overrides gitignore and safety)
    /// 5. If gitignore matches → Exclude
    /// 6. If safety matches → Exclude
    /// 7. Default → Include
    pub fn select_file(&self, rel_path: &RelPath) -> Selection {
        let path_str = rel_path.as_match_str();

        let matched_include = self.matches_include_rules(&path_str, rel_path);

        // Priority 1: If include patterns exist but file doesn't match any, exclude
        if self.has_includes && !matched_include {
            return Selection::Exclude;
        }

        // Priority 2: Path-specific includes override exclude
        // (e.g., `-I vendor/**/*.py` overrides `-X vendor`)
        if matched_include && self.matches_path_specific_include(&path_str) {
            return Selection::Include;
        }

        // Priority 3: Exclude patterns narrow down generic includes
        if let Some(ref exclude_globset) = self.exclude_globset {
            if exclude_globset.is_match(path_str.as_ref()) {
                return Selection::Exclude;
            }
        }

        // Priority 4: Generic include overrides gitignore and safety
        if matched_include {
            return Selection::Include;
        }

        // Priority 5: Gitignore rules (check each scoped layer)
        if self.matches_gitignore(&path_str, rel_path, false) {
            return Selection::Exclude;
        }

        // Priority 6: Safety preset
        if let Some(ref safety) = self.safety_preset {
            if safety.matches(path_str.as_ref()) {
                return Selection::Exclude;
            }
        }

        // Default: include if no rules matched
        Selection::Include
    }

    /// Select whether to include, exclude, or prune a directory
    ///
    /// Priority order:
    /// 1. .git → always prune
    /// 2. Gitignore → always prune (like rg/fd: gitignored dirs are never traversed)
    /// 3. Safety preset → always prune
    /// 4. Include patterns may keep dir alive (prevents -X from pruning)
    /// 5. Exclude patterns (-X) → prune
    /// 6. Default → include
    pub fn select_dir(&self, rel_path: &RelPath) -> Selection {
        let path_str = rel_path.as_match_str();

        // Priority 1: Always exclude .git directory
        if path_str == ".git" || path_str.starts_with(".git/") {
            return Selection::PruneDir;
        }

        // Priority 2: Path-specific includes override gitignore/safety.
        // e.g., `-I vendor/**/*.py` explicitly targets vendor/, so we must
        // not prune it even if gitignore or safety would normally do so.
        if self.dir_may_contain_path_specific_includes(&path_str) {
            return Selection::Include;
        }

        // Priority 3: Gitignore always prunes directories.
        // Like rg/fd, gitignored directories are never traversed regardless
        // of generic include patterns. Users can opt out with --use-gitignore=never.
        if self.matches_gitignore(&path_str, rel_path, true) {
            return Selection::PruneDir;
        }

        // Priority 4: Safety preset always prunes directories.
        // Users can opt out with --unsafe.
        if let Some(ref safety) = self.safety_preset {
            if safety.matches(path_str.as_ref()) || safety.matches(&format!("{}/", path_str)) {
                return Selection::PruneDir;
            }
        }

        // Priority 5: Check if this directory might contain files matching
        // any include patterns (including generic ones like `**/src/**`).
        // This prevents `-X` from pruning directories that might have matches.
        let may_contain_includes = self.dir_may_contain_includes(&path_str);
        if self.matches_include_rules(&path_str, rel_path) || may_contain_includes {
            return Selection::Include;
        }

        // Priority 6: Exclude patterns (-X)
        if let Some(ref exclude_globset) = self.exclude_globset {
            // For directory matching, try both with and without trailing slash
            if exclude_globset.is_match(path_str.as_ref())
                || exclude_globset.is_match(format!("{}/", path_str))
            {
                return Selection::PruneDir;
            }
        }

        // Default: don't prune directories - we need to check their contents
        Selection::Include
    }

    /// Check if a path matches any gitignore layer, respecting directory scoping.
    /// Each layer has a scope (relative dir prefix). A layer only applies to
    /// paths under its scope. Scope "" means root (applies to everything).
    fn matches_gitignore(&self, path_str: &str, rel_path: &RelPath, is_dir: bool) -> bool {
        for (scope, gitignore) in &self.gitignore_layers {
            // Check if path is under this layer's scope
            if !scope.is_empty() && !path_str.starts_with(&format!("{}/", scope)) {
                continue;
            }

            // For scoped layers, match against the path relative to the scope dir
            let match_path = if scope.is_empty() {
                rel_path.to_path_buf()
            } else {
                PathBuf::from(&path_str[scope.len() + 1..])
            };

            if gitignore.matched(&match_path, is_dir).is_ignore() {
                return true;
            }
        }
        false
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

    /// Check if a path matches a path-specific include pattern.
    ///
    /// Path-specific patterns are those that don't start with `**/` - they target
    /// specific directories (e.g., `vendor/**/*.py`). These are considered "explicit"
    /// and override exclude patterns, while generic patterns (`**/*.rs`) can be
    /// narrowed by exclude.
    fn matches_path_specific_include(&self, path_str: &str) -> bool {
        if let Some(ref include_globset) = self.include_globset {
            if !include_globset.is_match(path_str) {
                return false;
            }
            // Check if any matching include pattern is path-specific
            for pattern in &self.include_glob {
                if !pattern.starts_with("**/") {
                    // Build a single glob to test this specific pattern
                    if let Ok(glob) = Glob::new(pattern) {
                        if glob.compile_matcher().is_match(path_str) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// Recursively collect `.gitignore` files from subdirectories of root.
    /// The root's own `.gitignore` is excluded (already handled by the upward walk).
    fn collect_nested_gitignores(root: &Path) -> Vec<PathBuf> {
        let mut result = Vec::new();
        let mut stack = Vec::new();

        // Seed with direct children of root
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    // Skip .git directory itself
                    if entry.file_name() == ".git" {
                        continue;
                    }
                    stack.push(entry.path());
                }
            }
        }

        while let Some(dir) = stack.pop() {
            let gitignore_path = dir.join(".gitignore");
            if gitignore_path.exists() {
                result.push(gitignore_path);
            }

            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
                        && entry.file_name() != ".git"
                    {
                        stack.push(entry.path());
                    }
                }
            }
        }

        result
    }

    /// Check if a directory potentially contains files that match
    /// path-specific include patterns (patterns that don't start with `**/`).
    ///
    /// Path-specific patterns explicitly target directories (e.g., `vendor/**/*.py`)
    /// and can override gitignore/safety pruning for those specific directories.
    /// Generic patterns (`**/src/**`) cannot — they defer to gitignore/safety.
    fn dir_may_contain_path_specific_includes(&self, path_str: &str) -> bool {
        for pattern in &self.include_glob {
            // Skip generic patterns — they don't override gitignore/safety
            if pattern.starts_with("**/") {
                continue;
            }

            // Check if the pattern targets files inside this directory
            // e.g., pattern "vendor/**/*.py" and dir "vendor" or "vendor/lib1"
            if pattern.starts_with(&format!("{}/", path_str)) {
                return true;
            }

            // Check if this directory is on the path to the pattern's target
            let static_prefix = if let Some(wildcard_pos) = pattern.find('*') {
                &pattern[..wildcard_pos]
            } else {
                pattern.as_str()
            };

            if static_prefix.starts_with(&format!("{}/", path_str)) {
                return true;
            }

            if !static_prefix.is_empty() && format!("{}/", path_str).starts_with(static_prefix) {
                return true;
            }
        }
        false
    }

    /// Check if a directory potentially contains files that match any include rules
    /// (both generic and path-specific).
    ///
    /// Used after gitignore/safety checks to prevent `-X` from pruning directories
    /// that might contain included files.
    fn dir_may_contain_includes(&self, path_str: &str) -> bool {
        for pattern in &self.include_glob {
            // Patterns starting with **/ can match files in any directory
            if pattern.starts_with("**/") {
                return true;
            }

            // Check if the pattern targets files inside this directory
            // e.g., pattern "vendor/**/*.py" and dir "vendor" or "vendor/lib1"
            if pattern.starts_with(&format!("{}/", path_str)) {
                return true;
            }

            // Check if this directory is on the path to the pattern's target
            // e.g., pattern "__tests__/nested/deep.py" and dir "__tests__"
            // Extract the static prefix before the first wildcard
            let static_prefix = if let Some(wildcard_pos) = pattern.find('*') {
                &pattern[..wildcard_pos]
            } else {
                pattern.as_str()
            };

            // If the directory is on the path to the pattern's target
            // e.g., static_prefix="__tests__/nested/" and dir="__tests__"
            if static_prefix.starts_with(&format!("{}/", path_str)) {
                return true;
            }

            // If the directory is within the pattern's scope
            // e.g., pattern "vendor/**/*.py" (static_prefix="vendor/") and dir "vendor/lib1"
            if !static_prefix.is_empty() && format!("{}/", path_str).starts_with(static_prefix) {
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
        // Exclude narrows include: files matching both are excluded
        let spec = MatchSpec::new()
            .with_include_glob(vec!["**/*.rs".to_string()])
            .with_exclude_glob(vec!["**/test/**".to_string()]);

        let temp_dir = TempDir::new().unwrap();
        let engine = MatcherEngine::compile(&spec, temp_dir.path()).unwrap();

        let src_rs = RelPath::from_relative("src/main.rs");
        assert_eq!(engine.select_file(&src_rs), Selection::Include);

        // test_main.rs matches both include (**/*.rs) and exclude (**/test/**),
        // exclude narrows include, so it should be excluded
        let test_rs = RelPath::from_relative("test/test_main.rs");
        assert_eq!(engine.select_file(&test_rs), Selection::Exclude);
    }

    #[test]
    fn test_nested_gitignore_scoping() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create nested .gitignore: a/b/c/.gitignore with *.tmp
        std::fs::create_dir_all(root.join("a/b/c")).unwrap();
        std::fs::write(root.join("a/b/c/.gitignore"), "*.tmp\n").unwrap();

        let spec = MatchSpec::new().with_gitignore(true);
        let engine = MatcherEngine::compile(&spec, root).unwrap();

        // a/b/keep.tmp should NOT be excluded (not under a/b/c/)
        let keep = RelPath::from_relative("a/b/keep.tmp");
        assert_eq!(
            engine.select_file(&keep),
            Selection::Include,
            "a/b/keep.tmp should not be affected by a/b/c/.gitignore"
        );

        // a/b/c/remove.tmp SHOULD be excluded
        let remove = RelPath::from_relative("a/b/c/remove.tmp");
        assert_eq!(
            engine.select_file(&remove),
            Selection::Exclude,
            "a/b/c/remove.tmp should be excluded by a/b/c/.gitignore"
        );
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
