use crate::terminal::detect::{TerminalDetector, TerminalMode};

/// Terminal capabilities and features
pub struct TerminalCapabilities {
    detector: TerminalDetector,
    #[allow(dead_code)]
    width: Option<usize>,
}

impl TerminalCapabilities {
    pub fn new() -> Self {
        let detector = TerminalDetector::new();
        let width = Self::detect_width();

        Self { detector, width }
    }

    #[allow(dead_code)]
    pub fn with_detector(detector: TerminalDetector) -> Self {
        let width = Self::detect_width();
        Self { detector, width }
    }

    /// Get terminal width
    #[allow(dead_code)]
    pub fn width(&self) -> usize {
        self.width.unwrap_or(80)
    }

    /// Check if we should display animations
    #[allow(dead_code)]
    pub fn supports_animation(&self) -> bool {
        self.detector.is_tty() && !Self::is_fun_disabled()
    }

    /// Check if fun mode is disabled
    #[allow(dead_code)]
    fn is_fun_disabled() -> bool {
        if let Ok(val) = std::env::var("NO_FUN") {
            !val.is_empty() && val != "0"
        } else {
            false
        }
    }

    /// Check if we should use emoji
    #[allow(dead_code)]
    pub fn supports_emoji(&self) -> bool {
        self.detector.is_tty() && self.detector.supports_unicode()
    }

    /// Check if we should use colors
    #[allow(dead_code)]
    pub fn supports_colors(&self) -> bool {
        self.detector.should_use_colors()
    }

    /// Check if we should use Unicode tree characters
    pub fn supports_unicode_trees(&self) -> bool {
        self.detector.supports_unicode()
    }

    /// Get the tree branch characters based on capabilities
    pub fn tree_chars(&self) -> TreeChars {
        if self.supports_unicode_trees() {
            TreeChars::unicode()
        } else {
            TreeChars::ascii()
        }
    }

    /// Get the progress bar characters based on capabilities
    #[allow(dead_code)]
    pub fn progress_chars(&self) -> ProgressChars {
        if self.detector.supports_unicode() {
            ProgressChars::unicode()
        } else {
            ProgressChars::ascii()
        }
    }

    /// Detect terminal width
    fn detect_width() -> Option<usize> {
        // Try to get terminal size using terminal_size crate would be better,
        // but for now we'll use a simple approach
        if let Ok(cols) = std::env::var("COLUMNS") {
            cols.parse().ok()
        } else {
            // Default terminal width
            Some(80)
        }
    }

    /// Get the output mode
    #[allow(dead_code)]
    pub fn output_mode(&self) -> TerminalMode {
        self.detector.output_mode()
    }
}

impl Default for TerminalCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

/// Tree drawing characters
#[derive(Debug, Clone)]
pub struct TreeChars {
    pub branch: &'static str,
    pub last_branch: &'static str,
    pub vertical: &'static str,
    pub empty: &'static str,
}

impl TreeChars {
    pub fn unicode() -> Self {
        Self {
            branch: "├─ ",
            last_branch: "└─ ",
            vertical: "│  ",
            empty: "   ",
        }
    }

    pub fn ascii() -> Self {
        Self {
            branch: "|-- ",
            last_branch: "`-- ",
            vertical: "|   ",
            empty: "    ",
        }
    }
}

/// Progress bar characters
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProgressChars {
    pub filled: &'static str,
    pub empty: &'static str,
    pub filled_md: &'static str,
    pub empty_md: &'static str,
}

impl ProgressChars {
    pub fn unicode() -> Self {
        Self {
            filled: "█",
            empty: "░",
            filled_md: "▰",
            empty_md: "▱",
        }
    }

    pub fn ascii() -> Self {
        Self {
            filled: "#",
            empty: "-",
            filled_md: "#",
            empty_md: "-",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_chars_unicode() {
        let caps = TerminalCapabilities::new();

        if caps.supports_unicode_trees() {
            let chars = caps.tree_chars();
            assert_eq!(chars.vertical, "│   ");
            assert_eq!(chars.branch, "├── ");
            assert_eq!(chars.last_branch, "└── ");
            assert_eq!(chars.empty, "    ");
        }
    }
}
