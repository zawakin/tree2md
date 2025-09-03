use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek};
use std::path::Path;

use crate::util::format::{TruncateType, TruncationInfo};

/// Result of probing a file for binary/text characteristics
#[derive(Debug)]
pub struct ProbeResult {
    pub is_binary: bool,
    pub is_utf8: bool,
    pub sample_len: usize,
}

/// Error type for read operations
#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Binary(u64),
    NonUtf8,
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::Io(e) => write!(f, "{}", e),
            ReadError::Binary(sz) => {
                write!(f, "Binary file ({})", crate::util::format::format_size(*sz))
            }
            ReadError::NonUtf8 => write!(f, "File is not valid UTF-8 text"),
        }
    }
}

impl std::error::Error for ReadError {}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> Self {
        ReadError::Io(e)
    }
}

/// Probe a file to determine if it's binary or text
pub fn probe_file(path: &Path, max_probe: usize) -> io::Result<ProbeResult> {
    let mut file = File::open(path)?;
    let meta_len = file.metadata().ok().map(|m| m.len()).unwrap_or(0);
    let probe_size = max_probe.min(meta_len as usize);

    let mut probe_buf = vec![0u8; probe_size];
    let n = file.read(&mut probe_buf)?;
    probe_buf.truncate(n);

    // Check for binary characteristics
    // A file is considered binary if it contains null bytes or too many control characters
    let has_null = probe_buf.contains(&0);
    let control_chars = probe_buf
        .iter()
        .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
        .count();

    // Consider binary if has null bytes or more than 10% control characters
    let is_binary = has_null || (control_chars > n / 10);

    // Check UTF-8 validity
    let is_utf8 = std::str::from_utf8(&probe_buf).is_ok();

    Ok(ProbeResult {
        is_binary,
        is_utf8,
        sample_len: n,
    })
}

/// Check if a file is too large based on size limit
pub fn is_too_large(path: &Path, max_size: u64) -> bool {
    match path.metadata() {
        Ok(meta) => meta.len() > max_size,
        Err(_) => false,
    }
}

/// Read text content from a file with optional byte and line limits
pub fn read_text_prefix(
    path: &Path,
    max_bytes: Option<usize>,
    max_lines: Option<usize>,
) -> Result<(String, TruncationInfo), ReadError> {
    let mut file = File::open(path)?;
    let meta_len = file.metadata().ok().map(|m| m.len()).unwrap_or(0);

    // First probe for binary
    let probe_result = probe_file(path, 8192).map_err(ReadError::Io)?;
    if probe_result.is_binary {
        return Err(ReadError::Binary(meta_len));
    }

    // Rewind to start
    file.seek(std::io::SeekFrom::Start(0))?;
    let mut reader = BufReader::new(file);

    let total_bytes = meta_len as usize;
    let mut total_lines = 0usize;
    let mut result_bytes = Vec::new();
    let mut buf = Vec::with_capacity(4096);

    let mut shown_lines = 0usize;
    let mut shown_bytes = 0usize;
    let mut truncated = false;
    let mut truncate_type = TruncateType::None;

    loop {
        buf.clear();
        let read_n = reader.read_until(b'\n', &mut buf)?;
        if read_n == 0 {
            break;
        }

        total_lines = total_lines.saturating_add(1);

        if truncated {
            // Once truncated, just count remaining lines without processing
            if let Some(max_l) = max_lines {
                if total_lines >= max_l * 2 {
                    // Stop counting after reaching double the max lines
                    break;
                }
            }
            continue;
        }

        // Check byte limit
        if let Some(max_b) = max_bytes {
            if shown_bytes + buf.len() > max_b {
                let remaining = max_b.saturating_sub(shown_bytes);
                if remaining > 0 {
                    let mut cut = remaining.min(buf.len());
                    // Ensure we don't cut in the middle of a UTF-8 character
                    while cut > 0 && std::str::from_utf8(&buf[..cut]).is_err() {
                        cut -= 1;
                    }
                    if cut > 0 {
                        result_bytes.extend_from_slice(&buf[..cut]);
                        shown_bytes += cut;
                        shown_lines += buf[..cut].iter().filter(|&&b| b == b'\n').count();
                    }
                }
                truncated = true;
                truncate_type = if max_lines.is_some() {
                    TruncateType::Both
                } else {
                    TruncateType::Bytes
                };
                continue;
            }
        }

        // Check line limit
        if let Some(max_l) = max_lines {
            if shown_lines >= max_l {
                truncated = true;
                truncate_type = if max_bytes.is_some() {
                    TruncateType::Both
                } else {
                    TruncateType::Lines
                };
                continue;
            }
        }

        result_bytes.extend_from_slice(&buf);
        shown_bytes += buf.len();
        shown_lines += 1;
    }

    let mut content = String::from_utf8(result_bytes).map_err(|_| ReadError::NonUtf8)?;

    // Ensure content ends with newline
    if !content.ends_with('\n') {
        content.push('\n');
    }

    Ok((
        content,
        TruncationInfo {
            truncated,
            total_lines,
            total_bytes,
            shown_lines,
            shown_bytes,
            truncate_type,
        },
    ))
}

/// Check if a file is likely binary based on extension
pub fn is_binary_extension(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            let ext_lower = ext_str.to_lowercase();

            // Common binary extensions
            let binary_extensions = [
                "exe", "dll", "so", "dylib", "a", "lib", "o", "obj", "png", "jpg", "jpeg", "gif",
                "bmp", "ico", "webp", "svg", "pdf", "zip", "tar", "gz", "bz2", "xz", "7z", "rar",
                "mp3", "mp4", "avi", "mkv", "mov", "wav", "flac", "ttf", "otf", "woff", "woff2",
                "eot", "db", "sqlite", "sqlite3", "pyc", "pyo", "class", "jar", "war", "lock",
                "sum",
            ];

            return binary_extensions.contains(&ext_lower.as_str());
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_probe_text_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "Hello, world!\nThis is a test.").unwrap();

        let result = probe_file(&path, 8192).unwrap();
        assert!(!result.is_binary);
        assert!(result.is_utf8);
    }

    #[test]
    fn test_probe_binary_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.bin");
        fs::write(&path, b"Hello\0World").unwrap();

        let result = probe_file(&path, 8192).unwrap();
        assert!(result.is_binary);
    }

    #[test]
    fn test_is_too_large() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        fs::write(&path, "small file").unwrap();

        assert!(!is_too_large(&path, 100));
        assert!(is_too_large(&path, 5));
    }

    #[test]
    fn test_binary_extensions() {
        assert!(is_binary_extension(Path::new("test.exe")));
        assert!(is_binary_extension(Path::new("image.png")));
        assert!(is_binary_extension(Path::new("archive.zip")));
        assert!(!is_binary_extension(Path::new("code.rs")));
        assert!(!is_binary_extension(Path::new("text.txt")));
    }

    #[test]
    fn test_read_text_with_limits() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n";
        fs::write(&path, content).unwrap();

        // Test line limit
        let (text, info) = read_text_prefix(&path, None, Some(3)).unwrap();
        assert_eq!(text, "Line 1\nLine 2\nLine 3\n");
        assert!(info.truncated);
        assert_eq!(info.shown_lines, 3);

        // Test byte limit
        let (text, info) = read_text_prefix(&path, Some(10), None).unwrap();
        assert!(text.len() <= 11); // Allow for newline at end
        assert!(info.truncated);
    }
}
