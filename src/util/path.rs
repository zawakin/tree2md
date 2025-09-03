use std::path::{Path, PathBuf};

/// Calculate display path relative to the display root
/// This is simplified from the old version since we no longer support multiple display modes
pub fn calculate_display_path(resolved_path: &Path, display_root: &Path) -> PathBuf {
    // Always use relative paths from display root
    pathdiff::diff_paths(resolved_path, display_root).unwrap_or_else(|| resolved_path.to_path_buf())
}

/// Normalize a path string (remove ./, //, etc)
#[cfg(test)]
pub fn normalize_path_string(path: &str) -> String {
    // Handle root path special case
    if path == "/" {
        return "/".to_string();
    }

    let mut result = String::new();
    let mut prev_was_slash = false;
    let starts_with_slash = path.starts_with('/');

    for ch in path.chars() {
        if ch == '/' || ch == '\\' {
            if !prev_was_slash && (!result.is_empty() || starts_with_slash) {
                result.push('/');
            }
            prev_was_slash = true;
        } else {
            prev_was_slash = false;
            result.push(ch);
        }
    }

    // Remove leading ./
    if result.starts_with("./") {
        result.drain(0..2);
    }

    // Remove trailing slash unless it's root
    if result.len() > 1 && result.ends_with('/') {
        result.pop();
    }

    if result.is_empty() {
        ".".to_string()
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_string() {
        assert_eq!(normalize_path_string("./foo/bar"), "foo/bar");
        assert_eq!(normalize_path_string("foo//bar"), "foo/bar");
        assert_eq!(normalize_path_string("foo/bar/"), "foo/bar");
        assert_eq!(normalize_path_string("./"), ".");
        assert_eq!(normalize_path_string("/"), "/");
    }

    #[test]
    fn test_normalize_path_string_absolute_edges() {
        // Absolute paths with multiple slashes
        assert_eq!(normalize_path_string("//var///log/"), "/var/log");
        assert_eq!(normalize_path_string("///"), "/");
        assert_eq!(normalize_path_string("/foo//bar///baz/"), "/foo/bar/baz");

        // Note: On Unix, backslashes are not path separators, they're regular characters
        // The normalize function handles '/' and '\\' as separators, so this will be normalized
        assert_eq!(normalize_path_string(r".\src\main.rs"), "src/main.rs");

        // Mixed slashes
        assert_eq!(normalize_path_string("foo//bar///"), "foo/bar");
        assert_eq!(normalize_path_string("./foo//./bar"), "foo/./bar");

        // Edge cases
        assert_eq!(normalize_path_string(""), ".");
        assert_eq!(normalize_path_string("."), ".");
        assert_eq!(normalize_path_string(".."), "..");
        assert_eq!(normalize_path_string("./.."), "..");
    }

    #[test]
    fn test_calculate_display_path() {
        let resolved = PathBuf::from("/home/user/project/src/main.rs");
        let display_root = PathBuf::from("/home/user/project");

        let result = calculate_display_path(&resolved, &display_root);
        assert_eq!(result, PathBuf::from("src/main.rs"));
    }
}
