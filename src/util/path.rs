use crate::cli::DisplayPathMode;
use std::path::{Path, PathBuf};

/// Calculate display path based on mode and display root
pub fn calculate_display_path(
    resolved_path: &Path,
    display_mode: &DisplayPathMode,
    display_root: &Path,
    original_input: Option<&str>,
    strip_prefixes: &[String],
) -> PathBuf {
    let mut display_path = match display_mode {
        DisplayPathMode::Absolute => resolved_path.to_path_buf(),
        DisplayPathMode::Relative => pathdiff::diff_paths(resolved_path, display_root)
            .unwrap_or_else(|| resolved_path.to_path_buf()),
        DisplayPathMode::Input => {
            if let Some(input) = original_input {
                // Normalize the input path (remove redundant ./ and // etc)
                PathBuf::from(normalize_path_string(input))
            } else {
                // Fallback to relative if no original input
                pathdiff::diff_paths(resolved_path, display_root)
                    .unwrap_or_else(|| resolved_path.to_path_buf())
            }
        }
    };

    // Apply strip prefixes
    for prefix in strip_prefixes {
        if let Ok(stripped) = display_path.strip_prefix(prefix) {
            display_path = stripped.to_path_buf();
            break; // Only strip the first matching prefix
        }
    }

    display_path
}

/// Normalize a path string (remove ./, //, etc)
fn normalize_path_string(path: &str) -> String {
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

        let result = calculate_display_path(
            &resolved,
            &DisplayPathMode::Relative,
            &display_root,
            None,
            &[],
        );
        assert_eq!(result, PathBuf::from("src/main.rs"));

        let result = calculate_display_path(
            &resolved,
            &DisplayPathMode::Absolute,
            &display_root,
            None,
            &[],
        );
        assert_eq!(result, PathBuf::from("/home/user/project/src/main.rs"));

        let result = calculate_display_path(
            &resolved,
            &DisplayPathMode::Input,
            &display_root,
            Some("./src/main.rs"),
            &[],
        );
        assert_eq!(result, PathBuf::from("src/main.rs"));
    }
}
