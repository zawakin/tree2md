use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Result of probing a file for binary/text characteristics
#[derive(Debug)]
#[allow(dead_code)]
pub struct ProbeResult {
    pub is_binary: bool,
    pub is_utf8: bool,
    pub sample_len: usize,
}

/// Probe a file to determine if it's binary or text
pub fn probe_file(path: &Path, max_probe: usize) -> io::Result<ProbeResult> {
    let mut file = File::open(path)?;
    let meta_len = file.metadata().ok().map(|m| m.len()).unwrap_or(0);
    let probe_size = max_probe.min(meta_len as usize);

    let mut probe_buf = vec![0u8; probe_size];
    let n = file.read(&mut probe_buf)?;
    probe_buf.truncate(n);

    let has_null = probe_buf.contains(&0);
    let control_chars = probe_buf
        .iter()
        .filter(|&&b| b < 32 && b != 9 && b != 10 && b != 13)
        .count();

    let is_binary = has_null || (control_chars > n / 10);
    let is_utf8 = std::str::from_utf8(&probe_buf).is_ok();

    Ok(ProbeResult {
        is_binary,
        is_utf8,
        sample_len: n,
    })
}

/// Check if a file is too large based on size limit
#[allow(dead_code)]
pub fn is_too_large(path: &Path, max_size: u64) -> bool {
    match path.metadata() {
        Ok(meta) => meta.len() > max_size,
        Err(_) => false,
    }
}

/// Check if a file is likely binary based on extension
pub fn is_binary_extension(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            let ext_lower = ext_str.to_lowercase();

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
}
