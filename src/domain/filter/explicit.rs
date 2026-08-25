//! Explicit path lists supplied via `--paths-from` / `--files0-from`.
//!
//! Unlike `-I`/`-X`, entries here are literal paths — no glob
//! interpretation — so a path list produced by `find -print0`,
//! `git ls-files -z`, `fd -0`, or a `jq` pipeline can be piped straight in
//! without worrying about metacharacters in file names.

use std::collections::HashSet;
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

/// Set of explicitly listed paths, normalized relative to the scan root
/// (forward slashes, no leading `./`, no trailing `/`).
#[derive(Debug, Clone, Default)]
pub struct ExplicitPaths {
    /// Paths the user listed (files or directories).
    listed: HashSet<String>,
    /// Every proper ancestor directory of a listed path. These must be
    /// traversed (and shown) so listed entries are reachable in the tree.
    ancestors: HashSet<String>,
    /// The scan root itself was listed (`.`): everything is covered.
    root_listed: bool,
}

impl ExplicitPaths {
    /// Load a path list from `source` (`-` = stdin). `nul_separated` selects
    /// `\0` as the record separator instead of `\n`.
    ///
    /// Relative entries are interpreted relative to the current working
    /// directory (matching what `find`, `fd`, and `git ls-files` print), then
    /// re-based onto `root`. Entries outside `root` are skipped with a warning.
    pub fn load(source: &str, nul_separated: bool, root: &Path) -> io::Result<Self> {
        let mut raw = Vec::new();
        if source == "-" {
            io::stdin().read_to_end(&mut raw)?;
        } else {
            raw = std::fs::read(source).map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!("cannot read path list '{}': {}", source, e),
                )
            })?;
        }
        let text = String::from_utf8_lossy(&raw);
        let sep = if nul_separated { '\0' } else { '\n' };
        let cwd = std::env::current_dir()?;
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let mut set = Self::default();
        for entry in text.split(sep) {
            let entry = if nul_separated {
                entry
            } else {
                entry.strip_suffix('\r').unwrap_or(entry)
            };
            if entry.is_empty() {
                continue;
            }
            match rebase(entry, &cwd, &root) {
                Some(rel) => set.insert(&rel),
                None => eprintln!(
                    "Warning: path list entry '{}' is outside the scanned root; skipped",
                    entry
                ),
            }
        }
        Ok(set)
    }

    /// Build from already-root-relative paths (tests).
    #[allow(dead_code)]
    pub fn from_relative<I, S>(paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut set = Self::default();
        for p in paths {
            let p = p.as_ref().trim_matches('/');
            if !p.is_empty() {
                set.insert(p);
            }
        }
        set
    }

    fn insert(&mut self, rel: &str) {
        if rel.is_empty() {
            self.root_listed = true;
            return;
        }
        self.listed.insert(rel.to_string());
        let mut idx = 0;
        while let Some(pos) = rel[idx..].find('/') {
            self.ancestors.insert(rel[..idx + pos].to_string());
            idx += pos + 1;
        }
    }

    /// Listed directly, or beneath a listed directory.
    pub fn covers(&self, rel: &str) -> bool {
        if self.root_listed || self.listed.contains(rel) {
            return true;
        }
        let mut idx = 0;
        while let Some(pos) = rel[idx..].find('/') {
            if self.listed.contains(&rel[..idx + pos]) {
                return true;
            }
            idx += pos + 1;
        }
        false
    }

    /// A proper ancestor of some listed path.
    pub fn is_ancestor(&self, rel: &str) -> bool {
        self.ancestors.contains(rel)
    }
}

/// Resolve `entry` against `cwd` and express it relative to `root` using
/// forward slashes. Returns `None` when the path is not under `root`.
fn rebase(entry: &str, cwd: &Path, root: &Path) -> Option<String> {
    let path = Path::new(entry);
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    // Prefer the real path so symlinked cwd/root (e.g. macOS /tmp) line up;
    // fall back to lexical normalization for entries that don't exist.
    let abs = abs.canonicalize().unwrap_or_else(|_| normalize(&abs));
    let rel = abs.strip_prefix(root).ok()?;
    let s = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    // An empty string is the root itself and means "everything".
    Some(s)
}

fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_listed_and_descendants() {
        let e = ExplicitPaths::from_relative(["a/b.txt", "dir"]);
        assert!(e.covers("a/b.txt"));
        assert!(e.covers("dir"));
        assert!(e.covers("dir/x/y.rs"));
        assert!(!e.covers("a"));
        assert!(!e.covers("a/c.txt"));
        assert!(!e.covers("dir2/x"));
    }

    #[test]
    fn ancestors_are_tracked() {
        let e = ExplicitPaths::from_relative(["a/b/c.txt"]);
        assert!(e.is_ancestor("a"));
        assert!(e.is_ancestor("a/b"));
        assert!(!e.is_ancestor("a/b/c.txt"));
    }

    #[test]
    fn rebase_handles_dot_and_parent() {
        let root = Path::new("/r");
        assert_eq!(rebase("./x/y", root, root).as_deref(), Some("x/y"));
        assert_eq!(rebase("x/../z", root, root).as_deref(), Some("z"));
        assert_eq!(
            rebase("/r/sub/f", Path::new("/elsewhere"), root).as_deref(),
            Some("sub/f")
        );
        assert!(rebase("../out", root, root).is_none());
    }
}
