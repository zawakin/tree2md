use super::{ExplicitPaths, MatchSpec, RelPath};
use crate::cli::RuleKind;
use crate::safety::SafetyPreset;
use globset::{Glob, GlobMatcher};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::collections::HashMap;
use std::io;
use std::path::Path;
use std::sync::RwLock;

/// Selection decision for a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    Include,
    Exclude,
    PruneDir,
}

/// A single compiled rule with its kind, pattern and matcher.
struct CompiledRule {
    kind: RuleKind,
    pattern: String,
    matcher: GlobMatcher,
}

/// Compiled matcher engine.
///
/// The decision model is "last match wins":
/// * Rules are evaluated in argv order.
/// * For a given path, the *last* rule whose glob matches decides the
///   outcome (Include/Exclude).
/// * If no rule matches and includes are present, the path is excluded.
/// * If no rule matches and no includes are present, gitignore and safety
///   filters apply.
///
/// This gives users a single mental model: "write -I and -X in the order
/// you say them out loud; the later one carves out the earlier one."
pub struct MatcherEngine {
    rules: Vec<CompiledRule>,
    respect_gitignore: bool,
    /// Ancestor + root `.gitignore`, `.git/info/exclude`, and the global
    /// ignore file, scoped to the root.
    root_gitignore: Option<Gitignore>,
    /// Nested `.gitignore` files keyed by their directory's scope (`a/b`).
    /// Loaded as the walk enters each directory (`load_nested_gitignore`), so
    /// pruned subtrees are never scanned for them. Behind a lock because the
    /// engine is shared with the walker's `filter_entry`, which must be `Sync`.
    nested_gitignores: RwLock<HashMap<String, Gitignore>>,
    safety_preset: Option<SafetyPreset>,
    has_includes: bool,
    /// Literal path list from `--paths-from` / `--files0-from`. Acts as an
    /// include rule that precedes every argv rule, so `-X` can still carve
    /// entries out of the list.
    explicit: Option<ExplicitPaths>,
}

impl MatcherEngine {
    pub fn compile(spec: &MatchSpec, root: &Path) -> io::Result<Self> {
        let rules = spec
            .rules
            .iter()
            .map(|r| {
                let glob = Glob::new(&r.pattern).map_err(|e| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Invalid glob pattern '{}': {}", r.pattern, e),
                    )
                })?;
                Ok(CompiledRule {
                    kind: r.kind,
                    pattern: r.pattern.clone(),
                    matcher: glob.compile_matcher(),
                })
            })
            .collect::<io::Result<Vec<_>>>()?;

        let root_gitignore = if spec.respect_gitignore {
            build_root_gitignore(root)?
        } else {
            None
        };

        let safety_preset = if spec.use_safety_preset {
            Some(SafetyPreset::new())
        } else {
            None
        };

        Ok(Self {
            has_includes: spec.explicit_paths.is_some()
                || rules.iter().any(|r| r.kind == RuleKind::Include),
            rules,
            respect_gitignore: spec.respect_gitignore,
            root_gitignore,
            nested_gitignores: RwLock::new(HashMap::new()),
            safety_preset,
            explicit: spec.explicit_paths.clone(),
        })
    }

    /// Decide whether to include, exclude, or skip a file.
    pub fn select_file(&self, rel_path: &RelPath) -> Selection {
        let path_str = rel_path.as_match_str();
        let path_ref = path_str.as_ref();

        // Last-wins evaluation over CLI rules. An explicit path list acts as
        // the earliest include rule.
        let mut last: Option<RuleKind> = match &self.explicit {
            Some(e) if e.covers(path_ref) => Some(RuleKind::Include),
            _ => None,
        };
        for rule in &self.rules {
            if rule.matcher.is_match(path_ref) {
                last = Some(rule.kind);
            }
        }

        if let Some(kind) = last {
            return match kind {
                RuleKind::Include => Selection::Include,
                RuleKind::Exclude => Selection::Exclude,
            };
        }

        // No rule matched. If the user gave any -I, default-deny.
        if self.has_includes {
            return Selection::Exclude;
        }

        // Otherwise apply ambient filters.
        if self.matches_gitignore(&path_str, rel_path, false) {
            return Selection::Exclude;
        }
        if let Some(ref safety) = self.safety_preset {
            if safety.matches(path_ref) {
                return Selection::Exclude;
            }
        }
        Selection::Include
    }

    /// Decide whether to descend into, skip, or keep a directory.
    ///
    /// The directory decision walks the rules in order and tracks the *last*
    /// rule that "applies" to this directory. A rule applies if either:
    /// * It matches the directory path directly (typical for `-X build`), or
    /// * It could match files anywhere beneath this directory (typical for
    ///   `-I projects/alpha/**` when D is `projects` or `projects/alpha/...`).
    ///
    /// The final decision uses the kind of the last applicable rule. This
    /// keeps `-I A/** -X A/B` (prune the subtree) and `-X A -I A/B/**`
    /// (carve out a slice of an excluded subtree) both working naturally.
    pub fn select_dir(&self, rel_path: &RelPath) -> Selection {
        let path_str = rel_path.as_match_str();
        let path_ref = path_str.as_ref();

        // .git is never traversed.
        if path_ref == ".git" || path_ref.starts_with(".git/") {
            return Selection::PruneDir;
        }

        // Find the last "explicit" rule that applies to D. Explicit means:
        //   * Include rule whose static path prefix names D or its ancestor /
        //     descendant (e.g. `target/**` targets `target`, `target/sub`).
        //     A purely generic include like `**/*.rs` is NOT explicit — it's
        //     a soft "look anywhere" signal that must defer to gitignore /
        //     safety.
        //   * Exclude rule whose glob matches D directly.
        // An explicit decision overrides gitignore and safety. This is what
        // lets users carve specific subtrees out of an otherwise-ignored
        // location (`-X vendor -I vendor/**/*.py`) and conversely lets a
        // late `-X` prune a subtree an earlier `-I` opened up.
        // A listed directory (or one beneath / above a listed path) is an
        // explicit include: the user named it, so it overrides ambient filters.
        let mut last_explicit: Option<RuleKind> = match &self.explicit {
            Some(e) if e.covers(path_ref) || e.is_ancestor(path_ref) => Some(RuleKind::Include),
            _ => None,
        };
        for rule in &self.rules {
            let applies = match rule.kind {
                RuleKind::Include => path_specific_include_targets_dir(&rule.pattern, path_ref),
                RuleKind::Exclude => {
                    rule.matcher.is_match(path_ref)
                        || rule.matcher.is_match(format!("{}/", path_ref))
                }
            };
            if applies {
                last_explicit = Some(rule.kind);
            }
        }

        if let Some(kind) = last_explicit {
            return match kind {
                RuleKind::Include => Selection::Include,
                RuleKind::Exclude => Selection::PruneDir,
            };
        }

        // No explicit rule applies. Apply ambient filters.
        if self.matches_gitignore(&path_str, rel_path, true) {
            return Selection::PruneDir;
        }
        if let Some(ref safety) = self.safety_preset {
            if safety.matches(path_ref) || safety.matches(&format!("{}/", path_ref)) {
                return Selection::PruneDir;
            }
        }

        // Could any include rule — generic or specific — match files under
        // this directory? If so, we must descend so the per-file matcher can
        // do its work.
        let any_include_relates = self.rules.iter().any(|r| {
            r.kind == RuleKind::Include && include_rule_relates_to_dir(&r.pattern, path_ref)
        });
        if any_include_relates {
            return Selection::Include;
        }

        // No include relates and the user has -I active: descending here
        // can't produce any matching file. Prune for clarity and speed.
        if self.has_includes {
            return Selection::PruneDir;
        }

        Selection::Include
    }

    /// Load `<dir>/.gitignore`, if any, as a layer covering everything beneath
    /// `dir_rel`. Call when the walk enters a directory, before any of its
    /// entries are selected. A no-op when gitignore is not respected.
    pub fn load_nested_gitignore(&self, dir_rel: &RelPath, dir: &Path) -> io::Result<()> {
        if !self.respect_gitignore {
            return Ok(());
        }
        let gi_path = dir.join(".gitignore");
        if !gi_path.exists() {
            return Ok(());
        }
        let scope = dir_rel.as_match_str().into_owned();
        let mut builder = GitignoreBuilder::new(dir);
        builder.add(&gi_path);
        let gi = builder.build().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Failed to build gitignore for {}: {}", scope, e),
            )
        })?;
        self.nested_gitignores
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(scope, gi);
        Ok(())
    }

    fn matches_gitignore(&self, path_str: &str, rel_path: &RelPath, is_dir: bool) -> bool {
        if let Some(ref gitignore) = self.root_gitignore {
            if gitignore
                .matched(rel_path.to_path_buf(), is_dir)
                .is_ignore()
            {
                return true;
            }
        }

        // A nested layer applies to paths beneath its directory, so only the
        // ancestors of this path (each prefix ending before a `/`) can match.
        let nested = self
            .nested_gitignores
            .read()
            .unwrap_or_else(|e| e.into_inner());
        if nested.is_empty() {
            return false;
        }
        for (i, _) in path_str.match_indices('/') {
            if let Some(gitignore) = nested.get(&path_str[..i]) {
                if gitignore
                    .matched(Path::new(&path_str[i + 1..]), is_dir)
                    .is_ignore()
                {
                    return true;
                }
            }
        }
        false
    }
}

/// Does an *include* rule's pattern explicitly target a directory?
///
/// "Explicitly" means the pattern is path-specific (does not start with
/// `**/`) and the directory sits on the static path of the pattern — either
/// as an ancestor of the target (`target/**` named from dir `target`) or as
/// a descendant inside the target (`vendor/**/*.py` matched against dir
/// `vendor/lib1`). Generic patterns like `**/*.rs` are deliberately
/// excluded: they should not override gitignore / safety pruning, only path
/// shapes that the user actually typed.
fn path_specific_include_targets_dir(pattern: &str, dir_path: &str) -> bool {
    if pattern.starts_with("**/") || pattern == "**" {
        return false;
    }
    brace_expand(pattern)
        .iter()
        .any(|p| static_prefix_relates(p, dir_path))
}

/// Does this include rule's pattern relate to a directory `dir_path`?
///
/// "Relate" means either:
/// * The pattern uses `**/` at the head, which makes it match anywhere, or
/// * The pattern's static prefix is under `dir_path` (e.g. pattern
///   `projects/alpha/**` relates to dir `projects`), or
/// * `dir_path` is under the pattern's static prefix (e.g. pattern
///   `vendor/**/*.py` relates to dir `vendor/lib1`).
fn include_rule_relates_to_dir(pattern: &str, dir_path: &str) -> bool {
    if pattern.starts_with("**/") || pattern == "**" {
        return true;
    }
    brace_expand(pattern)
        .iter()
        .any(|p| static_prefix_relates(p, dir_path))
}

/// Shared prefix check used by both the "explicit target" and "any relate"
/// dir queries. `pattern` here is expected to be brace-free.
fn static_prefix_relates(pattern: &str, dir_path: &str) -> bool {
    let static_prefix = match pattern.find(['*', '?', '[']) {
        Some(idx) => &pattern[..idx],
        None => pattern,
    };
    let static_prefix = static_prefix.trim_end_matches('/');
    if static_prefix.is_empty() {
        return false;
    }
    if static_prefix == dir_path || static_prefix.starts_with(&format!("{}/", dir_path)) {
        return true;
    }
    if format!("{}/", dir_path).starts_with(&format!("{}/", static_prefix)) {
        return true;
    }
    false
}

/// Expand top-level `{a,b,c}` alternatives so the prefix-based dir relation
/// check works on patterns like `packages/foo-{a,b}/**/*.ts`. Nested braces
/// are expanded recursively. We do not interpret commas inside character
/// classes or escaped braces — globset itself handles those — this is a
/// purely static analysis helper.
fn brace_expand(pattern: &str) -> Vec<String> {
    let bytes = pattern.as_bytes();
    let n = bytes.len();

    // Find the first `{`. `{`, `}` and `,` are ASCII so byte-position scanning
    // is safe even when the pattern contains multi-byte UTF-8 (e.g. Japanese
    // directory names), and all `&pattern[..]` slices below cut at ASCII
    // boundaries so they're valid str slices.
    let i = match bytes.iter().position(|&b| b == b'{') {
        Some(p) => p,
        None => return vec![pattern.to_string()],
    };

    // Find the matching `}` accounting for nesting.
    let mut depth = 1i32;
    let mut j = i + 1;
    while j < n {
        match bytes[j] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        j += 1;
    }
    if depth != 0 {
        // Unbalanced — give up and treat the pattern as literal.
        return vec![pattern.to_string()];
    }

    let prefix = &pattern[..i];
    let inside = &pattern[i + 1..j];
    let suffix = &pattern[j + 1..];

    let mut results = Vec::new();
    for alt in split_top_level_commas(inside) {
        let combined = format!("{}{}{}", prefix, alt, suffix);
        results.extend(brace_expand(&combined));
    }
    results
}

fn split_top_level_commas(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut start = 0;
    let mut out = Vec::new();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'{' => depth += 1,
            b'}' => depth -= 1,
            b',' if depth == 0 => {
                out.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

/// Collect ancestor + root `.gitignore` files, `.git/info/exclude`, and the
/// global ignore file into the root-scoped Gitignore. Nested `.gitignore`
/// files are loaded during the walk (`MatcherEngine::load_nested_gitignore`).
fn build_root_gitignore(root: &Path) -> io::Result<Option<Gitignore>> {
    let mut root_builder = GitignoreBuilder::new(root);
    let mut has_root_patterns = false;

    let mut current = root;
    loop {
        let gi = current.join(".gitignore");
        if gi.exists() {
            root_builder.add(gi);
            has_root_patterns = true;
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    let git_info_exclude = root.join(".git/info/exclude");
    if git_info_exclude.exists() {
        root_builder.add(git_info_exclude);
        has_root_patterns = true;
    }

    if let Some(home) = dirs::home_dir() {
        let xdg = home.join(".config/git/ignore");
        let legacy = home.join(".gitignore");
        if xdg.exists() {
            root_builder.add(xdg);
            has_root_patterns = true;
        } else if legacy.exists() {
            root_builder.add(legacy);
            has_root_patterns = true;
        }
    }

    if !has_root_patterns {
        return Ok(None);
    }
    let gi = root_builder.build().map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Failed to build root gitignore: {}", e),
        )
    })?;
    Ok(Some(gi))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::FilterRule;
    use tempfile::TempDir;

    fn engine(spec: MatchSpec) -> MatcherEngine {
        let t = TempDir::new().unwrap();
        MatcherEngine::compile(&spec, t.path()).unwrap()
    }

    fn rule(kind: RuleKind, pattern: &str) -> FilterRule {
        FilterRule {
            kind,
            pattern: pattern.to_string(),
        }
    }

    #[test]
    fn last_wins_include_alone() {
        let spec = MatchSpec::new().with_include_glob(vec!["**/*.rs".into()]);
        let e = engine(spec);
        assert_eq!(
            e.select_file(&RelPath::from_relative("src/main.rs")),
            Selection::Include
        );
        assert_eq!(
            e.select_file(&RelPath::from_relative("README.md")),
            Selection::Exclude
        );
    }

    #[test]
    fn last_wins_exclude_after_path_include_narrows_subtree() {
        // The headline bug: -I projects/alpha/** -X build should prune build/.
        let spec = MatchSpec::new().with_rules(vec![
            rule(RuleKind::Include, "projects/alpha/**"),
            rule(RuleKind::Exclude, "build"),
        ]);
        let e = engine(spec);
        assert_eq!(
            e.select_file(&RelPath::from_relative("projects/alpha/index.ts")),
            Selection::Include
        );
        assert_eq!(
            e.select_file(&RelPath::from_relative("projects/alpha/build/out.js")),
            Selection::Exclude,
            "exclude after include must narrow"
        );
        assert_eq!(
            e.select_dir(&RelPath::from_relative("projects/alpha/build")),
            Selection::PruneDir
        );
    }

    #[test]
    fn last_wins_include_after_exclude_carves_out() {
        // Existing carve-out case: -X vendor -I vendor/**/*.py keeps .py files.
        let spec = MatchSpec::new().with_rules(vec![
            rule(RuleKind::Exclude, "vendor"),
            rule(RuleKind::Include, "vendor/**/*.py"),
        ]);
        let e = engine(spec);
        assert_eq!(
            e.select_file(&RelPath::from_relative("vendor/lib1/code.py")),
            Selection::Include
        );
        assert_eq!(
            e.select_file(&RelPath::from_relative("vendor/lib1/README.md")),
            Selection::Exclude
        );
        assert_eq!(
            e.select_dir(&RelPath::from_relative("vendor")),
            Selection::Include,
            "must descend into vendor to find .py"
        );
    }

    #[test]
    fn double_star_no_slash_is_fixed() {
        let spec = MatchSpec::new().with_rules(vec![
            rule(RuleKind::Include, "projects/alpha/**"),
            rule(RuleKind::Exclude, "**.generated.json"),
        ]);
        let e = engine(spec);
        assert_eq!(
            e.select_file(&RelPath::from_relative("projects/alpha/index.ts")),
            Selection::Include
        );
        assert_eq!(
            e.select_file(&RelPath::from_relative(
                "projects/alpha/data.generated.json"
            )),
            Selection::Exclude
        );
    }

    #[test]
    fn directory_with_no_relevant_include_is_pruned() {
        let spec = MatchSpec::new().with_include_glob(vec!["projects/alpha/**".into()]);
        let e = engine(spec);
        assert_eq!(
            e.select_dir(&RelPath::from_relative("projects/beta")),
            Selection::PruneDir
        );
        assert_eq!(
            e.select_dir(&RelPath::from_relative("projects/alpha")),
            Selection::Include
        );
        assert_eq!(
            e.select_dir(&RelPath::from_relative("projects")),
            Selection::Include
        );
    }

    #[test]
    fn git_dir_always_pruned() {
        let e = engine(MatchSpec::new());
        assert_eq!(
            e.select_dir(&RelPath::from_relative(".git")),
            Selection::PruneDir
        );
    }

    #[test]
    fn bare_exclude_anywhere() {
        let spec = MatchSpec::new().with_exclude_glob(vec!["archived".into()]);
        let e = engine(spec);
        assert_eq!(
            e.select_file(&RelPath::from_relative("projects/archived/old.ts")),
            Selection::Exclude
        );
        assert_eq!(
            e.select_dir(&RelPath::from_relative("projects/archived")),
            Selection::PruneDir
        );
    }

    #[test]
    fn nested_gitignore_scoped() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        std::fs::create_dir_all(root.join("a/b/c")).unwrap();
        std::fs::write(root.join("a/b/c/.gitignore"), "*.tmp\n").unwrap();
        let e = MatcherEngine::compile(&MatchSpec::new().with_gitignore(true), root).unwrap();

        // Nested layers are not scanned up front; they load as the walk
        // enters each directory.
        assert_eq!(
            e.select_file(&RelPath::from_relative("a/b/c/remove.tmp")),
            Selection::Include
        );
        for dir in ["a", "a/b", "a/b/c"] {
            e.load_nested_gitignore(&RelPath::from_relative(dir), &root.join(dir))
                .unwrap();
        }

        assert_eq!(
            e.select_file(&RelPath::from_relative("a/b/keep.tmp")),
            Selection::Include
        );
        assert_eq!(
            e.select_file(&RelPath::from_relative("a/b/c/remove.tmp")),
            Selection::Exclude
        );
        assert_eq!(
            e.select_file(&RelPath::from_relative("a/b/c/d/remove.tmp")),
            Selection::Exclude
        );
        // A sibling sharing the name prefix is outside the scope.
        assert_eq!(
            e.select_file(&RelPath::from_relative("a/b/cc/keep.tmp")),
            Selection::Include
        );
    }

    #[test]
    fn nested_gitignore_not_loaded_when_disabled() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        std::fs::create_dir_all(root.join("sub")).unwrap();
        std::fs::write(root.join("sub/.gitignore"), "*.tmp\n").unwrap();
        let e = MatcherEngine::compile(&MatchSpec::new().with_gitignore(false), root).unwrap();

        e.load_nested_gitignore(&RelPath::from_relative("sub"), &root.join("sub"))
            .unwrap();
        assert_eq!(
            e.select_file(&RelPath::from_relative("sub/x.tmp")),
            Selection::Include
        );
    }
}
