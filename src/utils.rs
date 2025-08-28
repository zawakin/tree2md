use crate::DisplayPathMode;
use glob::Pattern;
use std::io;
use std::path::{Path, PathBuf};

/// Parse a comma-separated list of file extensions
pub fn parse_ext_list(ext_string: &str) -> Vec<String> {
    ext_string
        .split(',')
        .map(|s| {
            let ext = s.trim().to_lowercase();
            if ext.starts_with('.') {
                ext
            } else {
                format!(".{}", ext)
            }
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// Compile glob patterns from strings
pub fn compile_patterns(pattern_strings: &[String]) -> io::Result<Vec<Pattern>> {
    let mut patterns = Vec::new();
    for pattern_str in pattern_strings {
        match Pattern::new(pattern_str) {
            Ok(pattern) => patterns.push(pattern),
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid glob pattern '{}': {}", pattern_str, e),
                ));
            }
        }
    }
    Ok(patterns)
}

/// Format bytes into human-readable size
#[allow(dead_code)] // Will be used for file size display feature
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Information about file truncation
#[derive(Debug)]
pub struct TruncationInfo {
    pub truncated: bool,
    pub total_lines: usize,
    pub total_bytes: usize,
    pub shown_lines: usize,
    pub shown_bytes: usize,
    pub truncate_type: TruncateType,
}

#[derive(Debug)]
pub enum TruncateType {
    None,
    Bytes,
    Lines,
    Both,
}

/// Generate a message describing how content was truncated
pub fn generate_truncation_message(info: &TruncationInfo) -> String {
    match info.truncate_type {
        TruncateType::Lines => {
            format!(
                "[Content truncated: showing first {} of {} lines]",
                info.shown_lines, info.total_lines
            )
        }
        TruncateType::Bytes => {
            format!(
                "[Content truncated: showing first {} of {} bytes]",
                info.shown_bytes, info.total_bytes
            )
        }
        TruncateType::Both => {
            format!(
                "[Content truncated: showing first {} of {} lines, {} of {} bytes]",
                info.shown_lines, info.total_lines, info.shown_bytes, info.total_bytes
            )
        }
        TruncateType::None => "[Content truncated]".to_string(),
    }
}

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
    let mut result = String::new();
    let mut prev_was_slash = false;

    for ch in path.chars() {
        if ch == '/' || ch == '\\' {
            if !prev_was_slash && !result.is_empty() {
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
    fn test_parse_ext_list() {
        let exts = parse_ext_list("go,py,.rs");
        assert_eq!(exts, vec![".go", ".py", ".rs"]);

        let exts = parse_ext_list(".md, .txt, rs");
        assert_eq!(exts, vec![".md", ".txt", ".rs"]);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(100), "100 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
        assert_eq!(format_size(1073741824), "1.0 GB");
    }

    #[test]
    fn test_generate_truncation_message() {
        let info = TruncationInfo {
            truncated: true,
            total_lines: 100,
            total_bytes: 5000,
            shown_lines: 50,
            shown_bytes: 2500,
            truncate_type: TruncateType::Lines,
        };
        assert_eq!(
            generate_truncation_message(&info),
            "[Content truncated: showing first 50 of 100 lines]"
        );

        let info = TruncationInfo {
            truncated: true,
            total_lines: 100,
            total_bytes: 5000,
            shown_lines: 50,
            shown_bytes: 2500,
            truncate_type: TruncateType::Both,
        };
        assert_eq!(
            generate_truncation_message(&info),
            "[Content truncated: showing first 50 of 100 lines, 2500 of 5000 bytes]"
        );
    }
}
