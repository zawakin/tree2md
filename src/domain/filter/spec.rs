use crate::cli::{Args, FilterRule, RuleKind};

/// Declarative specification of file matching rules.
///
/// The list of `-I` / `-X` rules is kept as a single ordered `Vec` so that
/// last-match-wins semantics can be evaluated by the engine. Each rule's
/// pattern is normalized at construction time so the engine sees only
/// well-formed globs.
#[derive(Debug, Clone)]
pub struct MatchSpec {
    /// Ordered filter rules from CLI (-I / -X), preserving argv order.
    pub rules: Vec<FilterRule>,

    /// Whether to respect gitignore files
    pub respect_gitignore: bool,

    /// Whether to apply safety presets (exclude sensitive files)
    pub use_safety_preset: bool,

    /// Whether pattern matching is case sensitive
    pub case_sensitive: bool,
}

impl Default for MatchSpec {
    fn default() -> Self {
        Self {
            rules: Vec::new(),
            respect_gitignore: false,
            use_safety_preset: true,
            case_sensitive: true,
        }
    }
}

impl MatchSpec {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Normalize a user-supplied glob pattern.
    ///
    /// Four things happen here:
    /// 1. Trailing `/` is stripped (`hoge/` and `hoge` are equivalent).
    /// 2. `**X` where X is not `/` is rewritten to `**/*X`. This is purely
    ///    a UX fix: globset treats `**.json` as a literal segment, not a
    ///    recursive match, so users who write `**.test.ts` silently get no
    ///    matches. We rewrite it to the intended form.
    /// 3. Patterns without `/` are anchored recursively: bare names like
    ///    `vendor` become `**/vendor/**`, file patterns like `*.rs` become
    ///    `**/*.rs`. This mirrors gitignore conventions so `-X build`
    ///    matches `build/` anywhere in the tree.
    /// 4. Path patterns whose *last segment* looks like a directory name
    ///    (no glob metacharacters and no `.`) get a `/**` appended so
    ///    `-I "projects/foo"` and `-I "projects/foo/"` both include the
    ///    contents of that directory. Without this step the pattern only
    ///    matched the literal path entry and produced an empty tree —
    ///    one of the original bug reports for the v0.10 series.
    pub fn normalize_pattern(pattern: &str) -> String {
        let pattern = pattern.strip_suffix('/').unwrap_or(pattern);
        let pattern = fix_double_star(pattern);

        if !pattern.contains('/') {
            return if !pattern.contains('*') && !pattern.contains('.') {
                format!("**/{}/**", pattern)
            } else {
                format!("**/{}", pattern)
            };
        }

        if looks_like_dir_path(&pattern) {
            return format!("{}/**", pattern);
        }
        pattern
    }

    /// Build a MatchSpec from CLI arguments.
    pub fn from_args(args: &Args, target_path: &std::path::Path) -> Self {
        let rules = args
            .filter_rules
            .iter()
            .map(|r| FilterRule {
                kind: r.kind,
                pattern: Self::normalize_pattern(&r.pattern),
            })
            .collect();

        let respect_gitignore = match args.use_gitignore {
            crate::cli::UseGitignoreMode::Always => true,
            crate::cli::UseGitignoreMode::Never => false,
            crate::cli::UseGitignoreMode::Auto => Self::is_inside_git_repo(target_path),
        };

        Self {
            rules,
            respect_gitignore,
            use_safety_preset: args.is_safe_mode(),
            case_sensitive: true,
        }
    }

    fn is_inside_git_repo(path: &std::path::Path) -> bool {
        let mut current = path;
        loop {
            if current.join(".git").exists() {
                return true;
            }
            match current.parent() {
                Some(parent) => current = parent,
                None => return false,
            }
        }
    }

    pub fn has_includes(&self) -> bool {
        self.rules.iter().any(|r| r.kind == RuleKind::Include)
    }

    /// Test helper: append include rules in order.
    #[allow(dead_code)]
    pub fn with_include_glob(mut self, patterns: Vec<String>) -> Self {
        for p in patterns {
            self.rules.push(FilterRule {
                kind: RuleKind::Include,
                pattern: Self::normalize_pattern(&p),
            });
        }
        self
    }

    /// Test helper: append exclude rules in order.
    #[allow(dead_code)]
    pub fn with_exclude_glob(mut self, patterns: Vec<String>) -> Self {
        for p in patterns {
            self.rules.push(FilterRule {
                kind: RuleKind::Exclude,
                pattern: Self::normalize_pattern(&p),
            });
        }
        self
    }

    /// Test helper: append rules preserving caller-specified order.
    #[allow(dead_code)]
    pub fn with_rules(mut self, rules: Vec<FilterRule>) -> Self {
        for r in rules {
            self.rules.push(FilterRule {
                kind: r.kind,
                pattern: Self::normalize_pattern(&r.pattern),
            });
        }
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

/// Does a `/`-containing pattern look like a bare directory path?
///
/// True when the final path segment contains no glob metacharacters and no
/// `.` (which would suggest a filename with an extension). We treat braces
/// (`{...}`) as wildcards because they expand to multiple alternatives, any
/// of which could be a file or a directory — leaving them un-expanded keeps
/// brace patterns like `packages/{a,b}` working as intended (`{a,b}` itself
/// has no `.` so users who really mean "all of these dirs" can append `/**`
/// explicitly; we only auto-expand when the segment is unambiguously a
/// literal name).
fn looks_like_dir_path(pattern: &str) -> bool {
    let last = match pattern.rsplit('/').next() {
        Some(s) if !s.is_empty() => s,
        _ => return false,
    };
    !last
        .chars()
        .any(|c| matches!(c, '.' | '*' | '?' | '[' | '{'))
}

/// Rewrite `**X` (where X is neither `/` nor `*`) as `**/*X`. This is the
/// pattern users intuitively write (`**.test.ts`) but globset treats as
/// a single literal segment. We patch it at the segment boundary only —
/// `**`, `**/`, `path/**`, `path/**/` are all left untouched.
///
/// Iterates over `chars()`, never `bytes()`, because patterns can contain
/// multi-byte UTF-8 (e.g. Japanese directory names) and a byte-level scan
/// would re-encode them as garbage.
fn fix_double_star(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len() + 4);
    let mut chars = pattern.chars().peekable();
    let mut prev: Option<char> = None;
    while let Some(c) = chars.next() {
        let at_segment_start = prev.is_none() || prev == Some('/');
        if at_segment_start && c == '*' && chars.peek() == Some(&'*') {
            // Consume the second '*'.
            chars.next();
            match chars.peek() {
                Some(&after) if after != '/' && after != '*' => {
                    out.push_str("**/*");
                }
                _ => {
                    out.push('*');
                    out.push('*');
                }
            }
            prev = Some('*');
            continue;
        }
        out.push(c);
        prev = Some(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_bare_name_to_recursive_dir() {
        assert_eq!(MatchSpec::normalize_pattern("vendor"), "**/vendor/**");
        assert_eq!(MatchSpec::normalize_pattern("__tests__"), "**/__tests__/**");
    }

    #[test]
    fn normalize_strips_trailing_slash() {
        assert_eq!(MatchSpec::normalize_pattern("hoge/"), "**/hoge/**");
        // Trailing slash on a path-shaped pattern marks it as a directory
        // and gets a recursive `/**` so contents are matched.
        assert_eq!(MatchSpec::normalize_pattern("src/lib/"), "src/lib/**");
        assert_eq!(MatchSpec::normalize_pattern("src/lib"), "src/lib/**");
    }

    #[test]
    fn normalize_dir_shaped_path_pattern() {
        // Path whose last segment looks like a bare directory name
        // (no `.`, no wildcards) is treated as a directory.
        assert_eq!(
            MatchSpec::normalize_pattern("projects/foo"),
            "projects/foo/**"
        );
        assert_eq!(
            MatchSpec::normalize_pattern("projects/foo/"),
            "projects/foo/**"
        );
        // A `.` in the last segment marks it as a filename — leave alone.
        assert_eq!(
            MatchSpec::normalize_pattern("projects/foo.ts"),
            "projects/foo.ts"
        );
        // Wildcard in the last segment — leave alone.
        assert_eq!(
            MatchSpec::normalize_pattern("projects/*.ts"),
            "projects/*.ts"
        );
        // Brace in the last segment — leave alone, brace expansion will
        // handle the alternatives.
        assert_eq!(
            MatchSpec::normalize_pattern("packages/{a,b}"),
            "packages/{a,b}"
        );
    }

    #[test]
    fn normalize_file_pattern_to_recursive() {
        assert_eq!(MatchSpec::normalize_pattern("*.rs"), "**/*.rs");
        assert_eq!(MatchSpec::normalize_pattern("foo.txt"), "**/foo.txt");
    }

    #[test]
    fn normalize_keeps_path_patterns() {
        assert_eq!(MatchSpec::normalize_pattern("src/*.go"), "src/*.go");
        assert_eq!(MatchSpec::normalize_pattern("**/*.md"), "**/*.md");
    }

    #[test]
    fn normalize_fixes_double_star_without_slash() {
        // The historical user-facing footgun.
        assert_eq!(MatchSpec::normalize_pattern("**.json"), "**/*.json");
        assert_eq!(MatchSpec::normalize_pattern("**.test.ts"), "**/*.test.ts");
        assert_eq!(
            MatchSpec::normalize_pattern("**.generated.json"),
            "**/*.generated.json"
        );
    }

    #[test]
    fn normalize_keeps_valid_double_star() {
        assert_eq!(MatchSpec::normalize_pattern("**/*.ts"), "**/*.ts");
        assert_eq!(MatchSpec::normalize_pattern("path/**"), "path/**");
        assert_eq!(MatchSpec::normalize_pattern("path/**/*.ts"), "path/**/*.ts");
        // Embedded `**`: `a/**/b/**.x` → `a/**/b/**/*.x`
        assert_eq!(MatchSpec::normalize_pattern("a/**/b/**.x"), "a/**/b/**/*.x");
    }

    #[test]
    fn has_includes_reflects_rules() {
        let spec = MatchSpec::new();
        assert!(!spec.has_includes());

        let spec = spec.with_include_glob(vec!["*.rs".into()]);
        assert!(spec.has_includes());

        let only_excl = MatchSpec::new().with_exclude_glob(vec!["build".into()]);
        assert!(!only_excl.has_includes());
    }
}
