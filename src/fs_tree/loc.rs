use crate::cli::LocMode;
use crate::content::io;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Line of Code counter
pub struct LocCounter {
    mode: LocMode,
    max_file_size: u64,
}

impl LocCounter {
    pub fn new(mode: LocMode) -> Self {
        Self {
            mode,
            // Don't count files larger than 10MB
            max_file_size: 10 * 1024 * 1024,
        }
    }

    /// Count lines in a file
    pub fn count_lines(&self, path: &Path) -> Option<usize> {
        if self.mode == LocMode::Off {
            return None;
        }

        // Check if file exists and is readable
        if !path.is_file() {
            return None;
        }

        // Use centralized I/O to check file size
        if io::is_too_large(path, self.max_file_size) {
            return None;
        }

        // Use centralized I/O to check if it's binary (check extension first for efficiency)
        if io::is_binary_extension(path) {
            return None;
        }

        // Probe file content to check if it's binary
        match io::probe_file(path, 8192) {
            Ok(probe) => {
                if probe.is_binary {
                    return None;
                }
            }
            Err(_) => return None,
        }

        // Perform the actual line count
        match self.mode {
            LocMode::Off => None,
            LocMode::Fast => self.count_lines_fast(path),
            LocMode::Accurate => self.count_lines_accurate(path),
        }
    }

    /// Fast line counting (just count newlines)
    fn count_lines_fast(&self, path: &Path) -> Option<usize> {
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);

        let mut count = 0;
        for _ in reader.lines() {
            count += 1;
            // Bail out if it's taking too long (>10000 lines)
            if count > 100_000 {
                break;
            }
        }

        Some(count)
    }

    /// Accurate line counting (skip blank lines and comments)
    fn count_lines_accurate(&self, path: &Path) -> Option<usize> {
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);

        let mut count = 0;
        for line in reader.lines() {
            if let Ok(line_str) = line {
                let trimmed = line_str.trim();
                // Skip blank lines
                if trimmed.is_empty() {
                    continue;
                }
                // Skip common comment patterns (simple heuristic)
                if trimmed.starts_with("//")
                    || trimmed.starts_with('#')
                    || trimmed.starts_with("/*")
                    || trimmed.starts_with("*")
                {
                    continue;
                }
                count += 1;
            }

            // Bail out if it's taking too long
            if count > 100_000 {
                break;
            }
        }

        Some(count)
    }

    // Removed is_binary_file method - now using centralized io::probe_file and io::is_binary_extension
}

impl Default for LocCounter {
    fn default() -> Self {
        Self::new(LocMode::Fast)
    }
}
