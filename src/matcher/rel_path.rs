use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// A normalized relative path that handles cross-platform differences
/// and non-UTF-8 paths gracefully.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelPath {
    inner: OsString,
}

impl RelPath {
    /// Create a RelPath from a path relative to a root.
    /// Returns None if the path is not under the root.
    pub fn from_root_rel<P: AsRef<Path>>(path: P, root: &Path) -> Option<Self> {
        let path = path.as_ref();
        
        // Try to strip the root prefix
        let relative = if let Ok(rel) = path.strip_prefix(root) {
            rel.as_os_str().to_owned()
        } else if let (Ok(canonical_path), Ok(canonical_root)) = 
            (path.canonicalize(), root.canonicalize()) {
            // If direct strip fails, try with canonicalized paths
            canonical_path.strip_prefix(&canonical_root).ok()?.as_os_str().to_owned()
        } else {
            return None;
        };

        // Convert to OsString for storage
        Some(Self { inner: relative })
    }

    /// Create a RelPath directly from a relative path
    pub fn from_relative<P: AsRef<Path>>(path: P) -> Self {
        Self {
            inner: path.as_ref().as_os_str().to_owned(),
        }
    }

    /// Get the path as a string for matching, with normalized separators.
    /// Uses forward slashes on all platforms for consistent matching.
    pub fn as_match_str(&self) -> Cow<'_, str> {
        let path_str = self.inner.to_string_lossy();
        
        // Always normalize backslashes to forward slashes for consistent matching
        // This handles paths that might contain backslashes on any platform
        if path_str.contains('\\') {
            Cow::Owned(path_str.replace('\\', "/"))
        } else {
            path_str
        }
    }

    /// Get the underlying OsStr
    pub fn as_os_str(&self) -> &OsStr {
        &self.inner
    }

    /// Convert to a PathBuf
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.inner)
    }

    /// Check if this path represents a directory based on trailing separator
    pub fn looks_like_dir(&self) -> bool {
        let s = self.inner.to_string_lossy();
        s.ends_with('/') || s.ends_with('\\')
    }
}

impl AsRef<OsStr> for RelPath {
    fn as_ref(&self) -> &OsStr {
        &self.inner
    }
}

impl AsRef<Path> for RelPath {
    fn as_ref(&self) -> &Path {
        Path::new(&self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::path::PathBuf;

    #[test]
    fn test_from_root_rel() {
        let root = Path::new("/home/user/project");
        let path = Path::new("/home/user/project/src/main.rs");
        
        let rel = RelPath::from_root_rel(path, root).unwrap();
        assert_eq!(rel.as_match_str(), "src/main.rs");
    }

    #[test]
    fn test_from_root_rel_not_under_root() {
        let root = Path::new("/home/user/project");
        let path = Path::new("/home/other/file.txt");
        
        assert!(RelPath::from_root_rel(path, root).is_none());
    }

    #[test]
    fn test_from_relative() {
        let rel = RelPath::from_relative("src/main.rs");
        assert_eq!(rel.as_match_str(), "src/main.rs");
    }

    #[test]
    fn test_windows_path_normalization() {
        // Create a path with backslashes using string manipulation
        // since PathBuf will use the native separator
        let rel = RelPath { inner: OsString::from("src\\main.rs") };
        // Even on non-Windows, we want consistent forward slashes
        let match_str = rel.as_match_str();
        assert!(!match_str.contains('\\'), "Backslashes should be normalized to forward slashes");
        assert_eq!(match_str, "src/main.rs");
    }

    #[test]
    fn test_looks_like_dir() {
        assert!(RelPath::from_relative("src/").looks_like_dir());
        assert!(!RelPath::from_relative("src/main.rs").looks_like_dir());
    }
}