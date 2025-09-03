use std::io;
use std::path::{Path, PathBuf};

/// Path validator that ensures paths stay within a restricted root
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PathValidator {
    restrict_root: Option<PathBuf>,
}

impl PathValidator {
    /// Create a new PathValidator with an optional restricted root
    #[allow(dead_code)]
    pub fn new(restrict_root: Option<PathBuf>) -> io::Result<Self> {
        let restrict_root = match restrict_root {
            Some(root) => {
                // Canonicalize the root path to resolve symlinks and get absolute path
                let canonical = root.canonicalize()?;
                Some(canonical)
            }
            None => None,
        };

        Ok(PathValidator { restrict_root })
    }

    /// Validate that a path is within the restricted root (if set)
    /// Returns Ok(canonical_path) if valid, Err if path escapes root
    #[allow(dead_code)]
    pub fn validate_path(&self, path: &Path) -> io::Result<PathBuf> {
        // Get canonical path
        let canonical = path.canonicalize()?;

        // If no restriction, path is valid
        let Some(ref root) = self.restrict_root else {
            return Ok(canonical);
        };

        // Check if path starts with the restricted root
        if canonical.starts_with(root) {
            Ok(canonical)
        } else {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "Path '{}' escapes restricted root '{}'",
                    path.display(),
                    root.display()
                ),
            ))
        }
    }

    /// Check if a path would be valid without canonicalizing
    /// Useful for checking paths that don't exist yet
    #[allow(dead_code)]
    pub fn would_be_valid(&self, path: &Path) -> bool {
        let Some(ref root) = self.restrict_root else {
            return true;
        };

        // Convert to absolute path if relative
        let abs_path = if path.is_relative() {
            std::env::current_dir()
                .ok()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|| path.to_path_buf())
        } else {
            path.to_path_buf()
        };

        // Basic check: does the path start with the root?
        // Note: This doesn't resolve symlinks, so it's less secure but works for non-existent paths
        abs_path.starts_with(root)
    }

    /// Get the restricted root if set
    #[allow(dead_code)]
    pub fn restrict_root(&self) -> Option<&Path> {
        self.restrict_root.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_no_restriction() {
        let validator = PathValidator::new(None).unwrap();

        // Any path should be valid when no restriction
        let current_dir = env::current_dir().unwrap();
        assert!(validator.validate_path(&current_dir).is_ok());
    }

    #[test]
    fn test_with_restriction() {
        let current_dir = env::current_dir().unwrap();
        let validator = PathValidator::new(Some(current_dir.clone())).unwrap();

        // Current directory should be valid
        assert!(validator.validate_path(&current_dir).is_ok());

        // Parent directory should be invalid
        if let Some(parent) = current_dir.parent() {
            assert!(validator.validate_path(parent).is_err());
        }
    }

    #[test]
    fn test_would_be_valid() {
        let current_dir = env::current_dir().unwrap();
        let validator = PathValidator::new(Some(current_dir.clone())).unwrap();

        // Relative path within current dir should be valid
        assert!(validator.would_be_valid(Path::new("src/main.rs")));

        // Absolute path outside should be invalid
        assert!(!validator.would_be_valid(Path::new("/etc/passwd")));
    }
}
